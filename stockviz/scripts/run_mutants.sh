#!/usr/bin/env bash
# r[impl talk.mutants] r[impl repo.scripts]
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"
exec cargo mutants "$@"
