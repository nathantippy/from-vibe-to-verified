#!/usr/bin/env bash
# r[impl repo.scripts] r[impl talk.ci.tiers]
# Main / nightly local path: default CI + strict tracey + optional mutants/fuzz.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$HERE/ci_local_default.sh"
export STOCKVIZ_TRACEY_STRICT=1
"$HERE/tracey_report.sh"
if command -v cargo-mutants >/dev/null 2>&1 || cargo mutants --version >/dev/null 2>&1; then
  "$HERE/run_mutants.sh" || echo "mutants: skipped or failed (optional on nightly)"
fi
if command -v cargo-fuzz >/dev/null 2>&1; then
  "$HERE/fuzz_csv.sh" -max_total_time=30 || echo "fuzz csv_parse: skipped or failed (optional on nightly)"
  "$HERE/fuzz_pipeline.sh" -max_total_time=30 || echo "fuzz my_target: skipped or failed (optional on nightly)"
fi
echo "release prep (strict tracey + fuzz): STOCKVIZ_RUN_FUZZ=1 ./scripts/quality_gate.sh"
