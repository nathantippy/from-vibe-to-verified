#!/usr/bin/env bash
# r[impl repo.scripts] r[impl talk.ci.tiers]
# PR / fast local path: fmt, clippy, nextest (no mutants, fuzz, or strict tracey).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$HERE/check_fmt.sh"
"$HERE/clippy_default.sh"
"$HERE/nextest_default.sh"
cargo test --doc
