# Shared `Arbitrary` testing (proptest + fuzz)

StockViz uses one module — [`src/test_inputs.rs`](../src/test_inputs.rs) — for **structured** random inputs. The same types feed:

| Tool | Oracle | Input |
|------|--------|--------|
| **proptest** | Property must hold | `arb::<T>()` via `proptest-arbitrary-interop` |
| **`my_target` fuzz** | Must not panic | `fuzz_target!(|input: PipelineFuzzInput|)` |
| **`csv_parse` fuzz** | Must not panic | `&[u8]` (unstructured; contrast deliberately) |

Tags: `r[test.arbitrary.shared]`, `r[test.arbitrary.proptest]`, `r[test.fuzz.pipeline]`.

---

## Mental model

```text
src/test_inputs.rs  (Arbitrary impls)
        │
        ├── proptest:  arb::<Sma50Series>()  → assert SMA == naive mean
        │
        └── libFuzzer: PipelineFuzzInput     → exercise_pipeline_input (no panic)
```

**Proptest** checks correctness on many generated values. **Fuzz** hunts crashes on structured (and mutated) inputs. They share generators, not oracles.

---

## Key types

| Type | Used by |
|------|---------|
| `ValidBarSeries` | Chart layout proptests |
| `AlignLayoutParams`, `BucketLayoutParams`, `WidthFillLayoutParams`, `PaneAlignLayoutParams` | Specific `r[test.proptest.*]` bounds |
| `Sma50Series`, `Sma150Series` | SMA reference proptests |
| `CsvRoundtripSeries` | CSV round-trip proptest |
| `TimeSeriesJsonBody` | `r[test.proptest.download.json]` dual byte-limit proptest |
| `ZoomSpanParams` | `r[test.proptest.zoom.limits]` zoom-in floor proptest |
| `ChartFuzzCtrl` | 64-byte control tail for chart/SMA parameters |
| `PipelineFuzzInput` | `my_target` + `pipeline_input_smoke` proptest |

---

## Proptest example

```rust
use proptest::prelude::*;
use proptest_arbitrary_interop::arb;
use stockviz::test_inputs::Sma50Series;

proptest! {
    #[test]
    fn sma_matches_naive(series in arb::<Sma50Series>()) {
        let closes = &series.closes;
        // ...
    }
}
```

---

## Fuzz example

[`fuzz/fuzz_targets/my_target.rs`](../fuzz/fuzz_targets/my_target.rs) decodes bytes into `PipelineFuzzInput` using the same `Arbitrary` / `from_legacy_bytes` logic as proptest:

```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(input) = PipelineFuzzInput::from_legacy_bytes(data) {
        exercise_pipeline_input(&input);
    }
});
```

Corpus bytes use legacy layout: `[mode u8][payload…][64-byte ctrl]`. Regenerate seeds with `./scripts/seed_pipeline_corpus.sh`.

---

## Why `csv_parse` stays `&[u8]`

Parser-only fuzz needs **hostile unstructured** bytes. Structured `PipelineFuzzInput` optimizes for chart/SMA pipeline depth; both run in `STOCKVIZ_RUN_FUZZ=1` quality gate.

---

## Commands

```bash
cargo test test_inputs proptest_          # proptest + arbitrary smoke
./scripts/fuzz_pipeline.sh -max_total_time=30
./scripts/fuzz_coverage.sh
./scripts/seed_pipeline_corpus.sh         # rewrite fuzz/corpus/my_target/seed_*
```

See also: [`docs/FUZZING.md`](FUZZING.md).
