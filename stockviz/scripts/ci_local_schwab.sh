#!/usr/bin/env bash
# r[repo.scripts] — schwab stub feature set (mirrors schwab CI job).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$HERE/clippy_schwab.sh"
"$HERE/nextest_schwab.sh"
