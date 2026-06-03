#!/usr/bin/env bash
# r[impl talk.fuzz.setup] r[impl repo.scripts] r[impl test.fuzz.pipeline]
# General pipeline fuzz (CSV/JSON + chart math). Requires nightly + cargo-fuzz.
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT/fuzz"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-nightly}"
FUZZ_ARGS=("$@")
if [[ ${#FUZZ_ARGS[@]} -eq 0 ]]; then
  FUZZ_ARGS=(-max_total_time=300 -max_len=4194304)
fi
exec cargo fuzz run my_target -- "${FUZZ_ARGS[@]}"
