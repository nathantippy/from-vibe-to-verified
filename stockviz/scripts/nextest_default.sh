#!/usr/bin/env bash
# r[impl repo.scripts] r[impl talk.nextest]
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"
exec cargo nextest run
