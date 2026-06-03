# StockViz — live demo runbook

One-page checklist for presenters. Full tag index: [`TALK_TAG_APPENDIX.md`](TALK_TAG_APPENDIX.md). Slide fixes: [`TALK_SLIDE_SYNC.md`](TALK_SLIDE_SYNC.md).

## Environment

```bash
export TWELVE_DATA_API_KEY=your_key   # download demo only
export RUST_LOG=info,stockviz=debug   # flexi_logger / tracing filter
export STOCKVIZ_TRACEY_STRICT=1       # fail tracey step if daemon/query breaks (main gate)
```

## Terminal layout (suggested)

| Pane | Command |
|------|---------|
| 1 | `bacon` or `bacon test` (optional background) |
| 2 | edits + `rg 'r\[impl'` / `rg 'r\[talk\.'` |
| 3 | `./scripts/ci_pr_fast.sh` or individual scripts |

## Demo order (45–60 min talk)

1. **Problem / stack** — open `docs/TALK_TAG_APPENDIX.md` § Stack (no build).
2. **Basic tests** — `rg 'r\[talk\.cargo\.test\.unit\]' src/talk_quiz.rs`; **sealed_test** — `rg 'r\[talk\.sealed\.test\]' src/talk_quiz.rs`
3. **Bacon** — `cat bacon.toml`
4. **Nextest** — `./scripts/nextest_default.sh`
5. **Tracey** — `stock_viz_spec.md`, `./scripts/tracey_report.sh`, `tracey web` (if installed)
6. **Coverage** — `./scripts/coverage.sh`
7. **Proptest** — `rg 'r\[verify test\.proptest\.sma\]' src/data.rs`; width fill: `rg 'r\[verify test\.proptest\.chart\.width\.fill\]' src/chart.rs` (see **Live proptest shrink** below)
8. **Mutants** — see [`MUTATION_TESTING.md`](MUTATION_TESTING.md); **do not** run `./scripts/run_mutants.sh` on stage (~15+ min). Show [`docs/mutants_after.txt`](mutants_after.txt) (0 missed) or one `Missed` line + `rg 'r\[verify data\.validation\]' src/data.rs` (2 min).
9. **Fuzz** — [`FUZZING.md`](FUZZING.md); show `fuzz/corpus/csv_parse/`; smoke only: `./scripts/fuzz_csv.sh -max_total_time=30` (do **not** run full 300s on stage)
10. **App** — `cargo run -- download AAPL && cargo run -- graph AAPL`
11. **Quiz** — `cargo nextest run -E 'test(talk_quiz)'` or `rg 'r\[talk\.quiz\.' src/talk_quiz.rs`

## Live proptest shrink (width fill)

`DEMO_WIDTH_PROPTEST_FAIL` in [`src/chart.rs`](../src/chart.rs) (`proptest_chart_width_fill`). **Never commit `true`.**

| Step | Action |
|------|--------|
| Green | `rg 'DEMO_WIDTH_PROPTEST_FAIL' src/chart.rs` → `false`; `bacon test` |
| Red | Set `DEMO_WIDTH_PROPTEST_FAIL: bool = true`, save; `bacon test` or `bacon coverage` |
| Read failure | Point at minimal failing `(n, width)` in output; optional `PROPTEST_VERBOSE=1` |
| Restore | Set back to `false`, save |

Story: old bug used one column per pixel (`cols == width`) instead of `min(bars, pixels)` per `r[gui.chart.width.fill]`.

## Quiz (presenter notes)

| Q | Demo |
|---|------|
| Q2 `should_panic` | CI uses correct `expected` + `#[ignore]`. To show failure live: wrong `expected` or un-ignore (see [`TALK_SLIDE_SYNC.md`](TALK_SLIDE_SYNC.md)). |
| Q5 `#[ignore]` | `cargo nextest run -- --ignored` runs `quiz_q5_ignored_expensive` |
| sealed_test | `cargo nextest run --test sealed_demo` |

## CI tiers (say this)

```bash
./scripts/ci_pr_fast.sh          # every PR — fmt, clippy, nextest
./scripts/ci_local_default.sh      # developer full default stack
STOCKVIZ_TRACEY_STRICT=1 ./scripts/quality_gate.sh   # main: strict tracey + ≥90% line cov
STOCKVIZ_RUN_FUZZ=1 ./scripts/quality_gate.sh      # optional: fuzz in gate (needs cargo-fuzz)
./scripts/ci_local_nightly.sh      # mutants + fuzz + strict tracey
```

## Tracy vs tracey

- **tracey** — spec coverage (`./scripts/tracey_report.sh`)
- **Tracy** — CPU profiler + `tracing` spans (`./scripts/run_tracy_profile.sh`)

Do not swap the names on slides.

## Before stage (tracey green)

```bash
cd /path/to/stockviz
tracey daemon
./scripts/tracey_report.sh
tracey query status
tracey query unmapped     # must be 0 unmapped code units (65/65 requirements)
tracey query uncovered    # should list nothing
tracey query untested     # should list nothing
STOCKVIZ_TRACEY_STRICT=1 ./scripts/tracey_report.sh
tracey web                # optional dashboard
```

If status still shows `0 of N covered`, restart the daemon after pulling latest code. Script-based talk tags are mirrored in `src/talk_tracey.rs`.

**Slop catch (unmapped):** comment out `// r[impl gui.chart.zoom]` on `zoom_in` in `src/chart.rs`, run `tracey query unmapped`, restore. See [`TRACEY_CODE_MAPPING.md`](TRACEY_CODE_MAPPING.md).

## Manual app smoke test

```bash
export TWELVE_DATA_API_KEY=your_key
cargo run -- download AAPL
cargo run -- graph AAPL
```
