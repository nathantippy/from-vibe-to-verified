#!/usr/bin/env bash
# r[repo.scripts]
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"
exec cargo clippy --all-targets -- -D warnings
