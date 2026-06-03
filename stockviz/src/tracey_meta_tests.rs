//! tracey verification helpers for repo-wide requirements (see `stock_viz_spec.md`).

use std::path::PathBuf;

// r[impl app.identity]
// r[verify app.identity]
#[test]
fn cargo_pkg_name_is_stockviz() {
    assert_eq!(env!("CARGO_PKG_NAME"), "stockviz");
}

// r[impl repo.scripts]
// r[verify repo.scripts]
#[test]
fn scripts_inventory_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts");
    for name in [
        "check_fmt.sh",
        "clippy_default.sh",
        "tracey_report.sh",
        "ci_local_default.sh",
        "ci_pr_fast.sh",
        "ci_local_nightly.sh",
        "quality_gate.sh",
        "verify_provider_exclusive.sh",
        "coverage.sh",
        "nextest_default.sh",
        "run_mutants.sh",
        "fuzz_csv.sh",
        "fuzz_pipeline.sh",
        "fuzz_coverage.sh",
        "seed_pipeline_corpus.sh",
    ] {
        let p = root.join(name);
        assert!(p.is_file(), "missing {p:?}");
    }
}

// r[impl build.provider]
// r[verify build.provider]
#[test]
fn default_feature_is_twelve_data() {
    #[cfg(not(feature = "twelve-data"))]
    compile_error!("default build should use twelve-data in talk demo");
}

// r[impl build.provider.exclusive]
// r[verify build.provider.exclusive]
#[test]
fn provider_exclusive_script_checks_both_features() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/verify_provider_exclusive.sh");
    let text = std::fs::read_to_string(&p).expect("verify_provider_exclusive.sh");
    assert!(
        text.contains("twelve-data") && text.contains("schwab"),
        "script should exercise mutually-exclusive provider build"
    );
}

// r[impl test.tracey.workflow]
// r[verify test.tracey.workflow]
#[test]
fn tracey_report_script_non_empty() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/tracey_report.sh");
    let n = std::fs::metadata(&p).expect("tracey_report.sh").len();
    assert!(n > 50, "script unexpectedly empty");
}

// r[impl test.tracey.coverage]
// r[verify test.tracey.coverage]
#[test]
fn spec_documents_tracey_coverage_rule() {
    let spec = include_str!("../stock_viz_spec.md");
    assert!(spec.contains("r[test.tracey.coverage]"));
    let mapping = include_str!("../docs/TRACEY_CODE_MAPPING.md");
    assert!(mapping.contains("unmapped"));
}

// r[impl test.fuzz.csv]

// r[impl talk.mutants]
// r[verify talk.mutants]
#[test]
fn mutation_testing_doc_exists() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/MUTATION_TESTING.md");
    let text = std::fs::read_to_string(&p).expect("MUTATION_TESTING.md");
    assert!(text.contains("cargo-mutants"));
    assert!(text.contains("49"));
}

// r[impl test.fuzz.csv]
// r[verify test.fuzz.csv]
#[test]
fn fuzzing_doc_exists() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/FUZZING.md");
    let text = std::fs::read_to_string(&p).expect("FUZZING.md");
    assert!(text.contains("cargo-fuzz"));
    assert!(text.contains("parse_csv_bytes"));
}

// r[impl test.tracey.coverage]
#[test]
fn tracey_mapping_doc_exists() {
    let doc = include_str!("../docs/TRACEY_CODE_MAPPING.md");
    assert!(doc.contains("tracey query unmapped"));
    assert!(doc.contains("Code unit mapping"));
}

// r[verify test.fuzz.csv]
#[test]
fn parse_csv_bytes_handles_arbitrary_input_without_panicking() {
    use crate::data::parse_csv_bytes;

    let cases: &[&[u8]] = &[
        b"\xff\xfe\x00",
        b"Date,Open,High,Low,Close,Volume\nx",
        &[0u8; 2048],
    ];
    for raw in cases {
        let _ = parse_csv_bytes(raw);
    }
}

// r[impl test.strategy]
// r[verify test.strategy]
#[test]
fn spec_lists_full_testing_stack() {
    let spec = include_str!("../stock_viz_spec.md");
    for tool in [
        "nextest", "tracey", "llvm-cov", "proptest", "mutants", "fuzz", "bacon",
    ] {
        assert!(spec.contains(tool), "missing {tool} in spec");
    }
}

// r[impl app.simple]
// r[verify app.simple]
#[test]
fn library_has_no_database_module() {
    let lib = include_str!("lib.rs");
    assert!(!lib.contains("mod database"));
    assert!(!lib.contains("sqlx"));
}

// r[impl app.deps]
// r[verify app.deps]
#[test]
fn cargo_toml_lists_core_deps() {
    let manifest = include_str!("../Cargo.toml");
    for dep in ["clap", "csv", "egui", "flexi_logger", "thiserror", "chrono"] {
        assert!(manifest.contains(dep), "missing dep {dep}");
    }
}

// r[impl app.overview]
// r[verify app.overview]
#[test]
fn cli_parses_download_and_graph() {
    use crate::cli::{Cli, Command};
    use clap::Parser;

    let g = Cli::try_parse_from(["stockviz", "graph", "x.csv"]).unwrap();
    assert!(matches!(g.command, Command::Graph { .. }));

    #[cfg(feature = "twelve-data")]
    {
        let d = Cli::try_parse_from(["stockviz", "download", "AAPL"]).unwrap();
        assert!(matches!(d.command, Command::Download { .. }));
    }
}

// r[impl cli]
// r[verify cli]
#[test]
fn cli_module_exports_commands() {
    let src = include_str!("cli.rs");
    assert!(src.contains("enum Command"));
}

// r[impl app.logging]
// r[verify app.logging]
#[test]
fn main_initializes_flexi_logger() {
    let main_rs = include_str!("main.rs");
    assert!(main_rs.contains("flexi_logger"));
    assert!(main_rs.contains("error!"));
}

// r[impl app.logging.tracing]
// r[verify app.logging.tracing]
#[test]
fn main_initializes_tracing_subscriber() {
    let main_rs = include_str!("main.rs");
    assert!(main_rs.contains("tracing_subscriber"));
}

// r[impl app.errors]
// r[verify app.errors]
#[test]
fn main_fatal_path_uses_log_error_and_exit_one() {
    let main_rs = include_str!("main.rs");
    assert!(main_rs.contains("error!"));
    assert!(main_rs.contains("process::exit(1)"));
}

// r[impl data.format]
// r[verify data.format]
#[test]
fn load_missing_csv_returns_err() {
    use crate::data::load_csv;
    let err = load_csv(std::path::Path::new("/nonexistent_stockviz_talk.csv")).unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty());
}

// r[impl talk.bacon]
// r[verify talk.bacon]
#[test]
fn bacon_toml_runs_nextest_job() {
    let text =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bacon.toml"))
            .expect("bacon.toml");
    assert!(text.contains("nextest"));
}

// r[impl talk.nextest]
// r[verify talk.nextest]
#[test]
fn nextest_script_invokes_cargo_nextest() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/nextest_default.sh");
    let text = std::fs::read_to_string(&p).expect("nextest_default.sh");
    assert!(text.contains("cargo nextest"));
}

// r[impl talk.llvm.cov]
// r[verify talk.llvm.cov]
#[test]
fn coverage_script_invokes_llvm_cov() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/coverage.sh");
    let text = std::fs::read_to_string(&p).expect("coverage.sh");
    assert!(text.contains("llvm-cov"));
}

// r[impl talk.mutants]
// r[verify talk.mutants]
#[test]
fn mutants_script_and_config_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("scripts/run_mutants.sh").is_file());
    let sh = std::fs::read_to_string(root.join("scripts/run_mutants.sh")).expect("run_mutants.sh");
    assert!(sh.contains("cargo mutants"));
    assert!(!sh.contains("cargo mutants run"));
    let toml = std::fs::read_to_string(root.join(".cargo/mutants.toml")).expect("mutants.toml");
    assert!(toml.contains("exclude_globs"));
}

// r[impl talk.fuzz.setup]
// r[verify talk.fuzz.setup]
#[test]
fn fuzz_csv_target_and_corpus_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = root.join("fuzz/fuzz_targets/csv_parse.rs");
    assert!(target.is_file(), "missing {target:?}");
    let corpus = root.join("fuzz/corpus/csv_parse");
    assert!(corpus.is_dir(), "missing corpus dir {corpus:?}");
    let seeds: Vec<_> = std::fs::read_dir(&corpus)
        .expect("corpus dir readable")
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .collect();
    assert!(
        seeds.len() >= 7,
        "expected at least 7 seed files in fuzz/corpus/csv_parse, got {}",
        seeds.len()
    );
}

// r[impl test.fuzz.pipeline]
// r[verify test.fuzz.pipeline]
#[test]
fn fuzz_pipeline_target_and_corpus_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target = root.join("fuzz/fuzz_targets/my_target.rs");
    assert!(target.is_file(), "missing {target:?}");
    let harness = root.join("src/fuzz_harness.rs");
    assert!(harness.is_file(), "missing {harness:?}");
    let corpus = root.join("fuzz/corpus/my_target");
    assert!(corpus.is_dir(), "missing corpus dir {corpus:?}");
    let seeds: Vec<_> = std::fs::read_dir(&corpus)
        .expect("corpus dir readable")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_file()).unwrap_or(false)
                && e.file_name().to_string_lossy().starts_with("seed_")
        })
        .collect();
    assert!(
        seeds.len() >= 5,
        "expected at least 5 seed_* files in fuzz/corpus/my_target, got {}",
        seeds.len()
    );
    assert!(root.join("fuzz/corpus/my_target/README.md").is_file());
    assert!(root.join("scripts/fuzz_pipeline.sh").is_file());
    assert!(root.join("scripts/fuzz_coverage.sh").is_file());
    let my_src = std::fs::read_to_string(&target).expect("my_target.rs");
    assert!(
        my_src.contains("PipelineFuzzInput"),
        "my_target must decode PipelineFuzzInput"
    );
    assert!(
        my_src.contains("exercise_pipeline_input"),
        "my_target must call exercise_pipeline_input"
    );
}

// r[impl test.arbitrary.shared]
// r[verify test.arbitrary.shared]
#[test]
fn arbitrary_shared_module_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("src/test_inputs.rs").is_file());
    assert!(root.join("docs/ARBITRARY_TESTING.md").is_file());
}

// r[impl talk.ci.tiers]
// r[verify talk.ci.tiers]
#[test]
fn ci_tier_scripts_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts");
    assert!(root.join("ci_pr_fast.sh").is_file());
    assert!(root.join("quality_gate.sh").is_file());
}

// r[impl talk.quality.gate]
// r[verify talk.quality.gate]
#[test]
fn quality_gate_sets_tracey_strict() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/quality_gate.sh");
    let text = std::fs::read_to_string(&p).expect("quality_gate.sh");
    assert!(text.contains("STOCKVIZ_TRACEY_STRICT=1"));
}

// r[impl test.fuzz.csv]
// r[verify test.fuzz.csv]
#[test]
fn quality_gate_supports_fuzz_opt_in() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/quality_gate.sh");
    let text = std::fs::read_to_string(&p).expect("quality_gate.sh");
    assert!(text.contains("STOCKVIZ_RUN_FUZZ"));
    assert!(text.contains("STOCKVIZ_FUZZ_SECONDS"));
}

// r[impl test.tracing]
// r[verify test.tracing]
#[test]
fn instrumented_hot_paths_in_source() {
    let data = include_str!("data.rs");
    let twelve = include_str!("download/twelve_data.rs");
    let gui = include_str!("gui/app.rs");
    assert!(data.contains("#[instrument"));
    assert!(twelve.contains("#[instrument"));
    assert!(gui.contains("#[instrument"));
}

// r[impl talk.stack.overview]
// r[verify talk.stack.overview]
#[test]
fn talk_appendix_documents_stack() {
    let appendix = include_str!("../docs/TALK_TAG_APPENDIX.md");
    assert!(appendix.contains("tracey"));
    assert!(appendix.contains("nextest"));
}

// r[impl test.tracey.workflow]
fn talk_tag_ids_from_spec() -> Vec<&'static str> {
    vec![
        "talk.stack.overview",
        "talk.cargo.test.unit",
        "talk.sealed.test",
        "talk.bacon",
        "talk.nextest",
        "talk.llvm.cov",
        "talk.mutants",
        "talk.fuzz.setup",
        "talk.ci.tiers",
        "talk.quality.gate",
        "talk.quiz.q1",
        "talk.quiz.q2",
        "talk.quiz.q3",
        "talk.quiz.q4",
        "talk.quiz.q5",
        "talk.quiz.q6",
        "talk.quiz.q7",
    ]
}

// r[impl test.tracey.workflow]
#[test]
fn every_talk_tag_has_impl_or_verify_in_repo() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let patterns = ["src", "scripts", "bacon.toml", "fuzz", "docs", ".cargo"];
    let mut corpus = String::new();
    for pat in patterns {
        let p = root.join(pat);
        if p.is_file() {
            corpus.push_str(&std::fs::read_to_string(&p).unwrap_or_default());
        } else if p.is_dir() {
            collect_rs_and_sh(&p, &mut corpus);
        }
    }
    for tag in talk_tag_ids_from_spec() {
        let needle_impl = format!("r[impl {tag}]");
        let needle_verify = format!("r[verify {tag}]");
        assert!(
            corpus.contains(&needle_impl) || corpus.contains(&needle_verify),
            "missing impl/verify for {tag}"
        );
    }
}

// r[impl test.tracey.coverage]
fn collect_rs_and_sh(dir: &std::path::Path, out: &mut String) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == "target" {
                continue;
            }
            collect_rs_and_sh(&path, out);
        } else if (path
            .extension()
            .is_some_and(|e| e == "rs" || e == "sh" || e == "toml")
            || path.file_name().is_some_and(|n| n == "bacon.toml"))
            && let Ok(s) = std::fs::read_to_string(&path)
        {
            out.push_str(&s);
        }
    }
}
