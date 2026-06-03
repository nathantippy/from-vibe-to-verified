//! StockViz library: data, download, chart helpers, GUI.
//!
//! Tracey: use **`// r[impl <tag>]`** / **`// r[verify <tag>]`** (space after `r[`, see [tracey](https://github.com/eikopf/tracey)).
//! Code-unit mapping guide: [`docs/TRACEY_CODE_MAPPING.md`](../docs/TRACEY_CODE_MAPPING.md).

#![forbid(unsafe_code)]

// r[impl app.identity]
// r[impl build.provider]
// Compile-time `twelve-data` (default) or `schwab` feature — see `download` module.

// r[impl build.provider.exclusive]
#[cfg(all(feature = "twelve-data", feature = "schwab"))]
compile_error!("Enable at most one provider: `twelve-data` (default) or `schwab`, not both.");

// r[impl app.simple]
// Zero persistent state except CSV files on disk; minimal dependency surface.

// r[impl talk.stack.overview]
// See docs/TALK_TAG_APPENDIX.md for the 2026 testing stack slide map.

// r[impl repo.scripts]
// Repository executable scripts live under `scripts/` (see `stock_viz_spec.md` §6).

// r[impl test.tracey.workflow]
// CI/local validation: `scripts/tracey_report.sh` runs `tracey query validate`.

// r[impl test.tracey.coverage]
// Spec requirements in `stock_viz_spec.md` are tagged with `r[...]`; Rust sources carry impl/verify comments.

// r[impl test.strategy]
// Full testing stack: scripts/, bacon.toml, nextest, tracey, llvm-cov, proptest, mutants, fuzz, kittest.

// r[impl app.deps]
// Dependency intent documented in Cargo.toml and stock_viz_spec.md §8.

// r[impl gui.chart]
pub mod chart;
// r[impl cli]
pub mod cli;
// r[impl data.format]
pub mod data;
// r[impl cli.download]
pub mod download;
// r[impl app.errors]
pub mod error;
// r[impl gui.core]
#[cfg(feature = "gui")]
pub mod gui;
// r[impl talk.cargo.test.unit]
pub mod talk_quiz;
// r[impl test.tracey.workflow]
pub mod talk_tracey;

// r[impl test.arbitrary.shared]
pub mod test_inputs;

// r[impl test.fuzz.pipeline]
pub mod fuzz_harness;

pub use error::{Error, Result};

// r[impl test.kittest.resize]
#[cfg(all(test, feature = "gui"))]
mod kittest_tests;

// r[impl test.tracey.workflow]
#[cfg(test)]
mod tracey_meta_tests;
