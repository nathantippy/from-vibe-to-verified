# StockViz — Talk demo tags (presentation only)

Tags here anchor the **“AI Agents + Rust Spec-Driven Testing”** talk. Product behavior lives in [`stock_viz_spec.md`](../stock_viz_spec.md).

r[talk.stack.overview]
This repository demonstrates the 2026 Rust spec-driven testing stack (see `docs/TALK_TAG_APPENDIX.md` § Stack table).

r[talk.cargo.test.unit]
Unit tests live in `#[cfg(test)]` modules and are stripped from release binaries.

r[talk.sealed.test]
`sealed_test` runs each test in an isolated subprocess with optional env vars in the attribute (`tests/sealed_demo.rs`; tracey `impl`/`verify` in `src/talk_quiz.rs`).

r[talk.bacon]
`bacon.toml` runs check, nextest, and clippy on save for instant local feedback.

r[talk.nextest]
`scripts/nextest_*.sh` wrap `cargo nextest run` for fast parallel test execution in CI.

r[talk.llvm.cov]
`scripts/coverage.sh` produces `lcov.info` via `cargo llvm-cov nextest`.

r[talk.mutants]
`scripts/run_mutants.sh` and `.cargo/mutants.toml` scope mutation testing to data/chart logic. See `docs/MUTATION_TESTING.md`.

r[talk.fuzz.setup]
`fuzz/` and `scripts/fuzz_csv.sh` demonstrate coverage-guided fuzzing of the CSV parser.

r[talk.ci.tiers]
PR uses a fast path; main/nightly uses strict tracey and optional quality gates.

r[talk.quality.gate]
`scripts/quality_gate.sh` enforces tracey strict mode and **≥90%** line coverage on main (`STOCKVIZ_MIN_COVERAGE`, default 90).

r[talk.quiz.q1]
`#[cfg(test)]` modules are omitted from release builds (see `src/talk_quiz.rs`).

r[talk.quiz.q2]
`#[should_panic(expected = "...")]` requires an exact substring match on the panic message.

r[talk.quiz.q3]
Tests may return `Result<(), E>` for clean error reporting without panicking.

r[talk.quiz.q4]
Unit tests in the same crate can call private functions via `use super::*`.

r[talk.quiz.q5]
`#[ignore]` tests run only with `cargo test -- --ignored` or `cargo nextest run -- --ignored`.

r[talk.quiz.q6]
Documentation examples in `///` blocks are compiled as tests under `cargo test --doc`.

r[talk.quiz.q7]
`RefCell` is idiomatic for interior mutability in single-threaded unit tests.
