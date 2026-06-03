//! Talk toolchain anchors for tracey (script/TOML entrypoints are not scanned as Rust).
//!
//! Each `r[impl talk.*]` here mirrors the real demo location documented in [`docs/TALK_TAG_APPENDIX.md`](../docs/TALK_TAG_APPENDIX.md).

// r[impl talk.sealed.test]
// Entry: tests/sealed_demo.rs — `#[sealed_test(env = [...])]`; tracey tags in src/talk_quiz.rs.

// r[impl talk.bacon]
// Entry: bacon.toml — background check / nextest / clippy on save.

// r[impl talk.nextest]
// Entry: scripts/nextest_default.sh, scripts/nextest_schwab.sh

// r[impl talk.llvm.cov]
// Entry: scripts/coverage.sh

// r[impl talk.mutants]
// Entry: scripts/run_mutants.sh, .cargo/mutants.toml

// r[impl talk.fuzz.setup]
// Entry: src/test_inputs.rs (Arbitrary), fuzz/fuzz_targets/csv_parse.rs (bytes),
// fuzz/fuzz_targets/my_target.rs (PipelineFuzzInput), scripts/fuzz_*.sh

// r[impl talk.ci.tiers]
// Entry: scripts/ci_pr_fast.sh, scripts/ci_local_nightly.sh

// r[impl talk.quality.gate]
// Entry: scripts/quality_gate.sh
