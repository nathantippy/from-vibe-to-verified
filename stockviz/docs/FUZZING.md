# Fuzzing (cargo-fuzz)

StockViz uses [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) and libFuzzer to hunt **panics** in parsers and chart/data pipeline code. Malformed input must return `Result::Err`, never crash.

Tags: `r[test.fuzz.csv]`, `r[test.fuzz.pipeline]`, `r[test.arbitrary.shared]`, `r[talk.fuzz.setup]`.

Structured generators: [`docs/ARBITRARY_TESTING.md`](ARBITRARY_TESTING.md) and [`src/test_inputs.rs`](../src/test_inputs.rs).

| Target | Script | Corpus |
|--------|--------|--------|
| **`csv_parse`** | [`scripts/fuzz_csv.sh`](../scripts/fuzz_csv.sh) | [`fuzz/corpus/csv_parse/`](../fuzz/corpus/csv_parse/) |
| **`my_target`** | [`scripts/fuzz_pipeline.sh`](../scripts/fuzz_pipeline.sh) | [`fuzz/corpus/my_target/`](../fuzz/corpus/my_target/) |

Coverage report (after corpus exists): [`scripts/fuzz_coverage.sh`](../scripts/fuzz_coverage.sh).

---

## Slim fuzz builds

Fuzz binaries link `stockviz` with `default-features = false` and **`twelve-data`** only (no **`gui`** / eframe). Faster builds and no egui warnings during fuzz. The desktop app still uses default features (`twelve-data` + `gui`).

---

## How this differs from other tests

| Tool | Question it answers |
|------|---------------------|
| **Unit / oracle tests** | Did we think of this bad CSV? |
| **proptest** | Do properties hold on generated valid-ish series? |
| **mutants** | Would tests fail if logic were wrong? |
| **fuzz** | Does *any* byte sequence panic the parser or pipeline? |

**llvm-cov** (nextest) can be green while fuzz still finds a panic — coverage records execution, not crash safety.

---

## `csv_parse` — parser aggressor

- **`data::parse_csv_bytes`** on arbitrary `&[u8]` (full input).
- Invalid UTF-8, bad headers, stress inputs (4 MiB cap via script defaults).

---

## `my_target` — structured pipeline harness

- **`PipelineFuzzInput`** decoded from fuzz bytes via **`from_legacy_bytes`** / **`Arbitrary`** ([`src/test_inputs.rs`](../src/test_inputs.rs); same types as proptest via `arb::<T>()`).
- Corpus bytes: `[mode u8][payload…][64-byte ChartFuzzCtrl]` — see [`fuzz/corpus/my_target/README.md`](../fuzz/corpus/my_target/README.md).
- Also runs `parse_csv_bytes` on encoded legacy bytes (parser stress).
- Payload decoded from fuzz bytes is capped at **65 KiB** (`MAX_PIPELINE_PAYLOAD`); JSON parsing uses the **pipeline** parser (`parse_time_series_json`, same 64 KiB cap). CLI download uses **`MAX_DOWNLOAD_JSON_BODY`** (1 MiB) separately — see `r[cli.download.twelvedata.api]`.
- Parsed JSON **`values`** at **5000 rows** (`MAX_TIME_SERIES_ROWS`, same as Twelve Data `outputsize`).
- On successful parse: chart/SMA/bucketing, zoom chain, **`write_csv` → re-parse** (≤5000 rows).

Implementation: [`src/fuzz_harness.rs`](../src/fuzz_harness.rs), [`fuzz/fuzz_targets/my_target.rs`](../fuzz/fuzz_targets/my_target.rs).

Regenerate seeds: `./scripts/seed_pipeline_corpus.sh`.

---

## Prerequisites

```bash
rustup toolchain install nightly
rustup component add llvm-tools-preview   # for fuzz_coverage.sh
cargo install cargo-fuzz --locked
```

Scripts set `RUSTUP_TOOLCHAIN=nightly` and run from `fuzz/` (except `fuzz_coverage.sh`).

---

## Commands

```bash
# Smoke (≈30s each, after compile)
./scripts/fuzz_csv.sh -max_total_time=30
./scripts/fuzz_pipeline.sh -max_total_time=30

# Full local run (300s + 4 MiB max input)
./scripts/fuzz_csv.sh
./scripts/fuzz_pipeline.sh

# Coverage profdata + line summary (my_target)
./scripts/fuzz_coverage.sh

# Minimize corpus after crashes (from fuzz/)
cd fuzz && cargo fuzz cmin my_target
cd fuzz && cargo fuzz cmin csv_parse

# Release prep (both targets when STOCKVIZ_RUN_FUZZ=1)
STOCKVIZ_RUN_FUZZ=1 STOCKVIZ_FUZZ_SECONDS=300 ./scripts/quality_gate.sh
```

`cargo +nightly fuzz coverage my_target` writes `fuzz/coverage/my_target/coverage.profdata` only; use **`fuzz_coverage.sh`** for human-readable output.

---

## Corpus

**`csv_parse`:** named seeds (`valid_minimal.csv`, `bad_header.csv`, …) plus optional local hash shards.

**`my_target`:** committed **`seed_*`** files only; libFuzzer hash shards are **gitignored**. Run `cmin` locally; copy crash artifacts into `seed_*` or new `seed_crash_*` names before commit.

---

## CI

- **PR / fast:** no fuzz (`ci_pr_fast.sh`).
- **Nightly:** optional 30s smoke for `csv_parse` and `my_target` (`ci_local_nightly.sh`).
- **Strict release prep:** `STOCKVIZ_RUN_FUZZ=1` in `quality_gate.sh` runs **both** targets.

---

## Related docs

- [`docs/TALK_TAG_APPENDIX.md`](TALK_TAG_APPENDIX.md) — slide map
- [`docs/MUTATION_TESTING.md`](MUTATION_TESTING.md) — mutation scope (chart math)
