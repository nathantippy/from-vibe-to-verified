#!/usr/bin/env bash
# r[impl talk.fuzz.setup] r[impl repo.scripts]
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT/fuzz"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-nightly}"
FUZZ_ARGS=("$@")
if [[ ${#FUZZ_ARGS[@]} -eq 0 ]]; then
  # 4 MiB input cap per r[test.fuzz.csv] stress guidance; 300s default run time.
  FUZZ_ARGS=(-max_total_time=300 -max_len=4194304)
fi
exec cargo fuzz run csv_parse -- "${FUZZ_ARGS[@]}"
