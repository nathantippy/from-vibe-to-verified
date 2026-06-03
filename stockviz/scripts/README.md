# StockViz — `scripts/` toolkit

Normative inventory and **`r[repo.scripts]`** are in [stock_viz_spec.md](../stock_viz_spec.md) §6.

All scripts are **`bash`** with **`set -euo pipefail`** and assume you run them from anywhere (they `cd` to the repo root via `git rev-parse --show-toplevel`).

## Rust toolchain

- **Stable** with components: `rustfmt`, `clippy`, `llvm-tools-preview`  
  `rustup component add rustfmt clippy llvm-tools-preview`

## Cargo plugins (examples)

| Tool | Typical install |
|------|------------------|
| **nextest** | `cargo install cargo-nextest --locked` or [upstream installers](https://nexte.st/docs/installation/) |
| **llvm-cov** | `cargo install cargo-llvm-cov --locked` |
| **cargo-mutants** | `cargo install cargo-mutants --locked` |
| **cargo-fuzz** | `cargo install cargo-fuzz --locked` |

Mutation config lives at **[.cargo/mutants.toml](../.cargo/mutants.toml)** (cargo-mutants default). **`./scripts/run_mutants.sh`** invokes **`cargo mutants`** (current CLI; there is no `run` subcommand).

## Fuzzing (`fuzz_csv.sh`, `fuzz_pipeline.sh`, `fuzz_coverage.sh`)

Full guide: [docs/FUZZING.md](../docs/FUZZING.md). Shared **`Arbitrary`** types: [docs/ARBITRARY_TESTING.md](../docs/ARBITRARY_TESTING.md).

| Script | Target | Role |
|--------|--------|------|
| **`fuzz_csv.sh`** | `csv_parse` | Arbitrary bytes → `parse_csv_bytes` |
| **`fuzz_pipeline.sh`** | `my_target` | `PipelineFuzzInput` (`Arbitrary`) pipeline |
| **`seed_pipeline_corpus.sh`** | — | Regenerate `fuzz/corpus/my_target/seed_*` |
| **`fuzz_coverage.sh`** | `my_target` | `cargo fuzz coverage` + **llvm-cov** summary/HTML |

- Usually requires a **nightly** toolchain for `libfuzzer-sys`.  
  Example: `rustup toolchain install nightly` then either set **`RUSTUP_TOOLCHAIN=nightly`** for the command or use `cargo +nightly fuzz …` if your setup prefers it.
- Fuzz run scripts execute inside **`fuzz/`**; **`fuzz_coverage.sh`** runs from repo root.
- Defaults when no extra args: **`-max_total_time=300`** and **`-max_len=4194304`** (4 MiB).
- Corpora: **`fuzz/corpus/csv_parse/`** (named seeds); **`fuzz/corpus/my_target/`** (`seed_*` + README; hash shards gitignored).

| Variable | Effect |
|----------|--------|
| **`STOCKVIZ_RUN_FUZZ=1`** | Run **both** fuzz targets in **`quality_gate.sh`** |
| **`STOCKVIZ_FUZZ_SECONDS`** | Override `-max_total_time` per target (default **300**) |
| **`STOCKVIZ_FUZZ_COVERAGE_HTML`** | HTML output dir for **`fuzz_coverage.sh`**; set to **`0`** to skip |

## tracey (`scripts/tracey_report.sh`)

StockViz uses **[eikopf/tracey](https://github.com/eikopf/tracey)** (spec ↔ implementation traceability). Configuration is **`.config/tracey/config.styx`** (spec **`stock-viz`**, Rust impl globs).

- If **`tracey`** is absent from **`PATH`**, the script **exits 0** and prints a skip message (CI-friendly).
- Otherwise it runs **`tracey query validate`**, **`tracey query status`**, and (when **`STOCKVIZ_TRACEY_STRICT=1`**) **`tracey query uncovered`**, **`untested`**, and **`unmapped`**, teeing logs under **`target/tracey/`** (`unmapped.log` must show **0 unmapped code units**). Validation errors cause **exit 1**. Transport/daemon failures exit **0** unless strict.
- Per-unit **`// r[impl <tag>]`** rules: [docs/TRACEY_CODE_MAPPING.md](../docs/TRACEY_CODE_MAPPING.md).
- **Strict gate prerequisite:** start the workspace daemon before strict runs: **`tracey daemon`** (see upstream docs). Then **`STOCKVIZ_TRACEY_STRICT=1 ./scripts/tracey_report.sh`**. **`tracey web`** opens the interactive UI.
- Talk-tool **`r[impl talk.*]`** mirrors for script-only entrypoints live in **`src/talk_tracey.rs`** (tracey scans Rust, not shell).

**Do not confuse** with **Tracy** (the profiler); see **`scripts/run_tracy_profile.sh`** and **`r[test.tracing]`** in the spec.

## Smart CI splitting (**`r[talk.ci.tiers]`**)

| When | Script |
|------|--------|
| Every PR / fast feedback | `./scripts/ci_pr_fast.sh` (fmt, clippy, nextest) |
| Developer full default | `./scripts/ci_local_default.sh` |
| Main branch gate | `STOCKVIZ_TRACEY_STRICT=1 ./scripts/quality_gate.sh` (≥**90%** line coverage) |
| Main gate + fuzz (opt-in) | `STOCKVIZ_RUN_FUZZ=1 STOCKVIZ_TRACEY_STRICT=1 ./scripts/quality_gate.sh` |
| Nightly local | `./scripts/ci_local_nightly.sh` |
| Schwab stub | `./scripts/ci_local_schwab.sh` |

| Strict coverage only | `STOCKVIZ_COVERAGE_STRICT=1 ./scripts/coverage.sh` (`--fail-under-lines 90`) |

Presenter index: [docs/TALK_TAG_APPENDIX.md](../docs/TALK_TAG_APPENDIX.md). Slide fixes: [docs/TALK_SLIDE_SYNC.md](../docs/TALK_SLIDE_SYNC.md).

## One-shot local CI

- Default + coverage + provider check: `./scripts/ci_local_default.sh`
- Schwab stub build: `./scripts/ci_local_schwab.sh`
