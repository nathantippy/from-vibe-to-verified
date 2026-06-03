#!/usr/bin/env bash
# Regenerate fuzz/corpus/my_target/seed_* (PipelineFuzzInput legacy bytes).
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"
cargo test test_inputs::tests::write_pipeline_corpus_seeds -- --ignored --exact
