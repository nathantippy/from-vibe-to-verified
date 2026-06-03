#!/usr/bin/env bash
# r[build.provider.exclusive] r[repo.scripts]
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"
tmp="$(mktemp)"
cleanup() { rm -f "$tmp"; }
trap cleanup EXIT

set +e
cargo check --features twelve-data,schwab 2>"$tmp"
ec=$?
set -e
if [[ "$ec" -eq 0 ]]; then
  echo "error: expected compile_error when both twelve-data and schwab are enabled"
  exit 1
fi
grep -q "at most one provider" "$tmp"
