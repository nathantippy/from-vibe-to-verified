#!/usr/bin/env bash
# r[impl repo.scripts] r[impl talk.llvm.cov]
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"
EXTRA=()
if [[ "${STOCKVIZ_COVERAGE_STRICT:-}" == 1 ]]; then
  EXTRA+=(--fail-under-lines "${STOCKVIZ_MIN_COVERAGE:-90}")
fi
exec cargo llvm-cov nextest --lcov --output-path lcov.info "${EXTRA[@]}"
