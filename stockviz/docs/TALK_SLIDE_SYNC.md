# Slide deck sync (StockViz repo truth)

Your slides live outside this repo. Use this table when updating the deck so commands and thresholds match CI.

| Slide topic | Use in slides |
|-------------|----------------|
| **Quality gate** | **≥90%** line coverage on main; `STOCKVIZ_TRACEY_STRICT=1 ./scripts/quality_gate.sh` |
| **CI tiers** | **PR:** `./scripts/ci_pr_fast.sh` (fmt, clippy, nextest, doc tests); tracey/coverage optional in GitHub Actions. **Main:** strict tracey + 90% cov via `quality_gate.sh`. **Nightly:** mutants + fuzz (`ci_local_nightly.sh`). |
| **Coverage command** | `./scripts/coverage.sh` → `lcov.info`; strict local: `STOCKVIZ_COVERAGE_STRICT=1 ./scripts/coverage.sh` |
| **Tracey CLI** | `tracey query status` / `unmapped` / `uncovered` / `untested`; `./scripts/tracey_report.sh` wraps validate + strict gates |
| **Tracey dashboard** | `tracey web --open` (after `tracey daemon`) |
| **Quiz Q5** | `cargo nextest run -- --ignored` (also valid: `cargo test -- --ignored`) |
| **Quiz Q2 (live fail)** | Repo: `expected = "attempt to divide by zero"` + `#[ignore]` for green CI. Slide trick `"division by zero"` still fails. Live fail demo: change `expected` to `"division by zero"`, then `cargo nextest run -- --ignored`. |
| **sealed_test** | `tests/sealed_demo.rs`; tracey: `r[talk.sealed.test]` in `src/talk_quiz.rs`. Run: `cargo nextest run --test sealed_demo` |
| **Tracy vs tracey** | **Tracy** = CPU profiler + `tracing`; **tracey** = spec tags (see runbook) |
| **Proptest width demo** | `DEMO_WIDTH_PROPTEST_FAIL` in `src/chart.rs` (`false` in repo); flip `true` + save → `bacon test`; ties to `r[gui.chart.width.fill]` |
| **Requirement count** | **65** tags (`stock_viz_spec.md` + `docs/talk_tags_spec.md`) |
| **Unmapped code units** | `tracey query unmapped` → **0** (strict main gate) |

## Stack one-liners (match conclusion slide)

```bash
cargo nextest run
./scripts/coverage.sh
./scripts/run_mutants.sh          # nightly / saved log on stage
./scripts/fuzz_csv.sh -max_total_time=300
cargo test --doc
STOCKVIZ_TRACEY_STRICT=1 ./scripts/tracey_report.sh
```

## sealed_test slide

- Crate: `sealed_test = "1.1"` in `[dev-dependencies]`
- Demo: `rg 'talk.sealed.test' src/talk_quiz.rs`
- Run: `cargo nextest run quiz_sealed`
