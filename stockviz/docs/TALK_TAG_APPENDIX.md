# StockViz — talk tag appendix

<!-- r[impl talk.stack.overview] -->

Master index for the **Rust Spec-Driven Testing (2026)** talk. Every `r[...]` tag in the repo, where to find it, and which slide block it supports.

Normative product spec: [`stock_viz_spec.md`](../stock_viz_spec.md). Talk-only spec: [`talk_tags_spec.md`](talk_tags_spec.md). Live commands: [`TALK_RUNBOOK.md`](TALK_RUNBOOK.md).

---

## How to grep

```bash
# Implementation site
rg 'r\[impl TAG\]' .

# Test / proof site (space after r[ required by tracey)
rg 'r\[verify TAG\]' .

# All talk demo anchors
rg 'r\[talk\.' .

# Requirement text in specs
rg 'r\[TAG\]' stock_viz_spec.md docs/talk_tags_spec.md
```

Replace `TAG` with e.g. `cli.download`, `gui.chart.zoom`, `talk.nextest`.

---

## Slide map

| Talk block | Say this (one line) | Primary tags | Open / run |
|------------|---------------------|--------------|------------|
| Open / vibe coding bottleneck | Review is the bottleneck; guardrails matter | — | slides only |
| Stack table | Six tools + bacon; tracey is the glue | `talk.stack.overview` | this file § Stack |
| Bash / CI tiers | PR fast, main strict | `talk.ci.tiers`, `repo.scripts` | `scripts/ci_pr_fast.sh`, `scripts/quality_gate.sh` |
| Basic testing (cargo test) | `#[cfg(test)]` strips from release | `talk.cargo.test.unit`, `talk.quiz.q1` | `src/talk_quiz.rs` |
| sealed_test | Isolated subprocess + env | `talk.sealed.test` | `src/talk_quiz.rs` |
| Bacon | Watch on save | `talk.bacon` | `bacon.toml`, `src/talk_tracey.rs` |
| Nextest | Parallel runner, flaky retry | `talk.nextest`, `build.provider` | `scripts/nextest_default.sh` |
| Tracey 1–8 | Spec ↔ impl ↔ verify | `test.tracey.*`, all `r[impl]` / `r[verify]` | `stock_viz_spec.md`, `scripts/tracey_report.sh` |
| Code mapping | Every fn/struct mapped | `test.tracey.coverage` | `tracey query unmapped`, [`TRACEY_CODE_MAPPING.md`](TRACEY_CODE_MAPPING.md) |
| llvm-cov | Lines executed ≠ correct | `talk.llvm.cov` | `scripts/coverage.sh` |
| Proptest 1–7 | Properties, not examples | `test.proptest.sma`, `test.arbitrary.shared`, `data.sma` | `src/test_inputs.rs`, `src/data.rs`, `src/chart.rs` |
| Arbitrary | One generator, two oracles | `test.arbitrary.shared`, `test.arbitrary.proptest` | `src/test_inputs.rs`, [`ARBITRARY_TESTING.md`](ARBITRARY_TESTING.md) |
| Mutants 1–4 | Kill mutants or tests are weak | `talk.mutants` | [`MUTATION_TESTING.md`](MUTATION_TESTING.md), `scripts/run_mutants.sh`, `.cargo/mutants.toml` |
| Fuzz 1–5 | Crashes and panics | `test.fuzz.csv`, `talk.fuzz.setup` | [`FUZZING.md`](FUZZING.md), `fuzz/corpus/csv_parse/`, `scripts/fuzz_csv.sh` |
| App demo (optional) | download → graph | `cli.download`, `gui.core` | `cargo run -- download AAPL` |
| Conclusion | Repeat stack table | `talk.stack.overview` | § Stack below |
| Quiz Q1–Q7 | Gotchas | `talk.quiz.q1` … `talk.quiz.q7` | `src/talk_quiz.rs` |

---

## Stack table (2026)

| Tool | StockViz entry | Tag |
|------|----------------|-----|
| cargo test / nextest | `scripts/nextest_*.sh` | `talk.nextest`, `build.provider` |
| tracey | `scripts/tracey_report.sh`, `.config/tracey/config.styx` | `test.tracey.workflow`, `test.tracey.coverage` |
| llvm-cov | `scripts/coverage.sh` | `talk.llvm.cov` |
| proptest | `src/test_inputs.rs`, `src/data.rs`, `src/chart.rs` | `test.proptest.sma`, `test.arbitrary.proptest` |
| cargo-mutants | `scripts/run_mutants.sh` | `talk.mutants` |
| cargo-fuzz | [`FUZZING.md`](FUZZING.md), `my_target` + `csv_parse` | `test.fuzz.pipeline`, `test.fuzz.csv`, `talk.fuzz.setup` |
| arbitrary | [`ARBITRARY_TESTING.md`](ARBITRARY_TESTING.md) | `test.arbitrary.shared` |
| bacon | `bacon.toml` | `talk.bacon` |
| egui_kittest | `src/kittest_tests.rs` | `test.kittest.resize`, `test.kittest.volume` |

---

## Script cheat sheet

| When | Command |
|------|---------|
| Every PR / fast local | `./scripts/ci_pr_fast.sh` |
| Developer default | `./scripts/ci_local_default.sh` |
| Main / release prep | `STOCKVIZ_TRACEY_STRICT=1 ./scripts/quality_gate.sh` |
| Full nightly local | `./scripts/ci_local_nightly.sh` |
| Schwab stub build | `./scripts/ci_local_schwab.sh` |

Talk block commands:

```bash
cargo nextest run
./scripts/coverage.sh
./scripts/run_mutants.sh
./scripts/fuzz_csv.sh
./scripts/tracey_report.sh
tracey query unmapped   # 0 unmapped (~193 code units)
tracey web    # if tracey installed
bacon test    # background watch
```

---

## Tracy vs tracey

| Name | What it is | StockViz |
|------|------------|----------|
| **tracey** | Spec traceability (eikopf/tracey) | `r[test.tracey.workflow]` |
| **Tracy** | CPU profiler | `r[test.tracing]`, `scripts/run_tracy_profile.sh` |

---

## Normative tags (`stock_viz_spec.md`)

| Tag | § | Impl (grep) | Verify (grep) | Demo |
|-----|---|-------------|---------------|------|
| `app.identity` | front | `src/main.rs`, `Cargo.toml` | `src/tracey_meta_tests.rs` | `cargo run -- --version` |
| `build.provider` | 0 | `src/lib.rs`, `src/download/mod.rs` | `src/tracey_meta_tests.rs` | `cargo build` / `--features schwab` |
| `build.provider.exclusive` | 0 | `src/lib.rs` | `src/tracey_meta_tests.rs` | `scripts/verify_provider_exclusive.sh` |
| `cli.download.twelvedata` | 0 | `src/download/twelve_data.rs` | same | `download --api-key` |
| `cli.download.twelvedata.api` | 0 | `src/download/twelve_data.rs` | same | mockito tests |
| `cli.download.schwab` | 0 | `src/download/mod.rs` | `schwab_tests` | schwab build only |
| `app.overview` | 1 | `src/main.rs`, `src/cli.rs` | `src/tracey_meta_tests.rs` | both subcommands |
| `gui.core` | 1 | `src/gui/mod.rs` | `src/kittest_tests.rs` | `graph` |
| `cli` | 2 | `src/cli.rs` | `cli.commands` verify | umbrella |
| `cli.commands` | 2 | `src/cli.rs` | `src/cli.rs` tests | `--help` |
| `cli.graph.path` | 2 | `src/data.rs`, `src/main.rs` | `data` + `cli` tests | `graph NOW` → `NOW.csv` |
| `cli.download` | 2 | `src/download/mod.rs` | twelve_data / schwab | `download` |
| `data.format` | 2 | `src/data.rs` | `src/data.rs` | CSV header |
| `data.validation` | 2 | `src/data.rs` | `src/data.rs` | bad CSV |
| `data.sma` | 2 | `src/data.rs` | `src/data.rs` | SMA 50 tests |
| `data.sma150` | 2 | `src/data.rs` | `src/data.rs` | SMA 150 tests |
| `gui.chart` | 3 | `src/gui/mod.rs`, `src/chart.rs` | `src/chart.rs`, kittest | chart window |
| `gui.chart.candles` | 3 | `src/gui/app.rs`, `src/chart.rs` | `src/chart.rs` | green/red candles + SMAs |
| `gui.chart.volume` | 3 | `src/gui/app.rs`, `src/chart.rs` | kittest + chart | volume pane |
| `gui.chart.timeaxis` | 3 | `src/gui/app.rs` | kittest | daily dates |
| `gui.chart.anchor` | 3 | `src/data.rs` | `src/data.rs` | right edge |
| `gui.chart.xticks` | 3 | `src/gui/app.rs` | kittest | sparse date labels |
| `gui.chart.yticks` | 3 | `src/chart.rs`, `src/gui/app.rs` | `src/chart.rs`, kittest | right price labels |
| `gui.chart.sma.legend` | 3 | `src/chart.rs`, `src/gui/app.rs` | `src/chart.rs`, kittest | SMA 50/150 legend |
| `gui.chart.zoom` | 3 | `src/gui/app.rs` | kittest | +/- buttons |
| `gui.chart.zoom.limits` | 3 | `src/chart.rs` | `src/chart.rs` | min 1 bar |
| `gui.chart.resize` | 3 | `src/gui/app.rs` | kittest | resize window |
| `gui.chart.sma.align` | 3 | `src/chart.rs`, `src/gui/app.rs` | `src/chart.rs` | SMA X = bucket center |
| `gui.chart.pane.align` | 3 | `src/chart.rs`, `src/gui/app.rs` | `src/chart.rs`, kittest | price/volume shared drawable width |
| `gui.chart.width.fill` | 3 | `src/chart.rs`, `src/gui/app.rs` | `src/chart.rs` | pane fills width |
| `app.errors` | 4 | `src/error.rs`, `src/main.rs` | `src/tracey_meta_tests.rs` | missing CSV |
| `app.logging` | 5 | `src/main.rs` | `src/tracey_meta_tests.rs` | `RUST_LOG` |
| `app.logging.tracing` | 5 | `src/main.rs`, `#[instrument]` | `src/tracey_meta_tests.rs` | Tracy |
| `test.strategy` | 6 | `scripts/README.md` | `src/tracey_meta_tests.rs` | script table |
| `test.kittest.resize` | 6 | `src/kittest_tests.rs` | same | zoom/resize |
| `test.kittest.volume` | 6 | `src/kittest_tests.rs` | same | volume colors |
| `test.proptest.download.json` | 6 | `src/download/twelve_data.rs` | `proptest_download_json` | Twelve Data JSON pipeline vs download limits |
| `test.proptest.sma` | 6 | `src/data.rs` | `proptest_sma` | SMA 50 properties |
| `test.proptest.sma150` | 6 | `src/data.rs` | `proptest_sma150` | SMA 150 properties |
| `test.proptest.sma.align` | 6 | `src/chart.rs` | `proptest_sma_align` | SMA/candle X |
| `test.proptest.pane.align` | 6 | `src/chart.rs` | `proptest_pane_align` | price/volume bucket X |
| `test.proptest.zoom.limits` | 6 | `src/chart.rs` | `proptest_zoom_limits` | 7-day zoom-in floor |
| `test.proptest.chart.width.fill` | 6 | `src/chart.rs` | `proptest_chart_width_fill` | last bar X extent |
| `test.fuzz.csv` | 6 | `fuzz/fuzz_targets/csv_parse.rs` | meta + fuzz | `fuzz_csv.sh` |
| `repo.scripts` | 6 | `scripts/*.sh` | `src/tracey_meta_tests.rs` | `scripts/` |
| `test.tracey.coverage` | 6 | `src/lib.rs` | `src/tracey_meta_tests.rs` | tracey status |
| `test.tracey.workflow` | 6 | `scripts/tracey_report.sh` | meta tests | validate |
| `test.tracing` | 6 | `src/main.rs`, spans | kittest verify | Tracy script |
| `app.simple` | 7 | `src/lib.rs` | meta tests | no DB |
| `app.deps` | 8 | `Cargo.toml` | meta tests | manifest |

---

## Talk tags (`docs/talk_tags_spec.md`)

| Tag | Slide | Impl | Verify |
|-----|-------|------|--------|
| `talk.stack.overview` | Stack | `docs/TALK_TAG_APPENDIX.md` | `src/tracey_meta_tests.rs` |
| `talk.cargo.test.unit` | Basic test | `src/talk_quiz.rs` | `talk_quiz` tests |
| `talk.sealed.test` | sealed_test | `tests/sealed_demo.rs`, `src/talk_tracey.rs` | `quiz_sealed_test_traced` + integration test |
| `talk.bacon` | Bacon | `bacon.toml` | meta (file exists) |
| `talk.nextest` | Nextest | `scripts/nextest_*.sh`, `src/talk_tracey.rs` | meta |
| `talk.llvm.cov` | Coverage | `scripts/coverage.sh`, `src/talk_tracey.rs` | meta |
| `talk.mutants` | Mutants | `scripts/run_mutants.sh`, `.cargo/mutants.toml`, `src/talk_tracey.rs` | meta |
| `talk.fuzz.setup` | Fuzz | `fuzz/`, `scripts/fuzz_csv.sh`, `src/talk_tracey.rs` | `test.fuzz.csv` |
| `talk.ci.tiers` | CI | `scripts/ci_pr_fast.sh`, `ci_local_nightly.sh`, `src/talk_tracey.rs` | meta |
| `talk.quality.gate` | Main gate | `scripts/quality_gate.sh`, `src/talk_tracey.rs` | meta |
| `talk.quiz.q1`–`q7` | Quiz | `src/talk_quiz.rs` | per-test `verify` |

---

## Quiz index

| Q | Topic | Test / symbol in `src/talk_quiz.rs` |
|---|--------|--------------------------------------|
| Q1 | `#[cfg(test)]` / release strip | `quiz_q1_cfg_test_runs` |
| Q2 | `should_panic` expected string | `quiz_q2_should_panic_demo` (ignored) |
| Q3 | `Result` in tests | `quiz_q3_result_err_path` |
| Q4 | private fn in unit test | `quiz_q4_private_helper` |
| Q5 | `#[ignore]` | `quiz_q5_ignored_expensive` |
| Q6 | doc tests | `talk_quiz::demo_add` rustdoc |
| Q7 | `RefCell` | `quiz_q7_refcell_counter` |
| sealed_test | isolated env | `quiz_sealed_env_isolated` |

Run: `cargo nextest run -E 'test(talk_quiz)'`

---

## Tracey script mirrors

Talk tools invoked from **bash/TOML** carry `// r[impl talk.*]` in [`src/talk_tracey.rs`](../src/talk_tracey.rs) so tracey’s Rust scanner sees them. The real scripts remain under `scripts/` and `bacon.toml`.

## Before talk (tracey)

See [`TALK_RUNBOOK.md`](TALK_RUNBOOK.md) § “Before stage (tracey green)”.

## Test hook (not user UI)

`__stockviz_volume__` label in the volume pane and `__stockviz_sma_legend__` in the price pane exist only under `#[cfg(test)]` for egui_kittest discovery. Documented here so presenters do not show them as product UI.
