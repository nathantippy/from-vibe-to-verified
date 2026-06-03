# Mutation testing (cargo-mutants)

StockViz uses [cargo-mutants](https://mutants.rs/) to find **weak tests**: tests that pass even when production logic is wrong.

Tag: `r[talk.mutants]`. Entry: [`scripts/run_mutants.sh`](../scripts/run_mutants.sh), [`.cargo/mutants.toml`](../.cargo/mutants.toml).

---

## Scope

| Included | Excluded (noise for the talk) |
|----------|-------------------------------|
| `src/chart.rs` | `src/gui/**` |
| `src/data.rs` | `src/main.rs` |
| `src/download/**` | `fuzz/**` |
| `src/talk_quiz.rs` | |

GUI and the binary are integration-heavy; mutating them slows the demo without teaching much. Chart **math** is tested in `chart.rs` without spinning egui.

---

## Baseline vs target

| Run | Missed | Caught | Notes |
|-----|--------|--------|-------|
| Before mutation-killing pass | 49 | 98 | Happy-path tests + llvm-cov green |
| After mutation-killing pass | 0 | 122 | Oracle, boundary, invariant, mock HTTP; 6 unviable |

Reproduce locally:

```bash
./scripts/run_mutants.sh
```

Read the summary line: `NN missed, MM caught, U unviable`. Open `mutants.out/` for per-mutant diffs if anything survives.

Latest clean run (talk slide): [`docs/mutants_after.txt`](mutants_after.txt) — `128 tested, 0 missed, 122 caught, 6 unviable`.

**Do not run the full suite live in a 16-person talk** (~15+ minutes). Use a saved log or show one `Missed` line + the test that kills it.

---

## Test categories that kill mutants

| Category | Example | Kills |
|----------|---------|-------|
| **Oracle** | `parse_rejects_nan_in_open_only` | `\|\|` → `&&` on finite checks |
| **Boundary** | `sma_50_exactly_fifty_closes` | `n < 50` → `n <= 50` |
| **Boundary** | `sma_150_exactly_one_fifty_closes` | `n < 150` → `n <= 150` |
| **Invariant** | `proptest_chart_buckets::buckets_partition_invariant` | Wrong partition / loop bounds in `buckets_for_width` |
| **Exact math** | `bars_fitting_width_exact` → 200 for 800/4 | Wrong `/`, `floor`, or `max(1)` |
| **Discriminating** | `demo_add(2, 3) == 5` | `+` → `*` (2+2 alone is not enough) |
| **Integration** | mockito + `download_to_csv` | `Ok(())` noops, URL encoding, HTTP/API errors |

**llvm-cov** can show high line coverage while mutants still miss: coverage records execution, not whether assertions would fail on wrong answers.

---

## Where to grep examples

```bash
rg 'r\[verify data\.validation\]' src/data.rs
rg 'r\[verify gui\.chart' src/chart.rs
rg 'r\[verify cli\.download' src/download/
rg 'r\[verify talk\.quiz\.q6\]' src/talk_quiz.rs
```

---

## Related docs

- [`docs/FUZZING.md`](FUZZING.md) — CSV parser panic fuzzing (complements mutants)
- [`docs/TALK_TAG_APPENDIX.md`](TALK_TAG_APPENDIX.md) — slide map, Mutants 1–4
- [`docs/TALK_RUNBOOK.md`](TALK_RUNBOOK.md) — live demo order
- [`docs/TRACEY_CODE_MAPPING.md`](TRACEY_CODE_MAPPING.md) — per-function `r[impl]` (orthogonal to mutants)
