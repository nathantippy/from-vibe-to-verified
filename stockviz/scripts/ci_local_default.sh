#!/usr/bin/env bash
# r[repo.scripts] — default features, mirrors main CI job (except fuzz/mutants).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$HERE/check_fmt.sh"
"$HERE/clippy_default.sh"
"$HERE/nextest_default.sh"
"$HERE/coverage.sh"
"$HERE/verify_provider_exclusive.sh"
"$HERE/tracey_report.sh"
