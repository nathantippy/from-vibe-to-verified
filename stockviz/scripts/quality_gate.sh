#!/usr/bin/env bash
# r[impl repo.scripts] r[impl talk.quality.gate] r[impl talk.ci.tiers]
# Main-branch quality gate: strict tracey + minimum line coverage.
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"

export STOCKVIZ_TRACEY_STRICT=1
./scripts/tracey_report.sh

MIN_COV="${STOCKVIZ_MIN_COVERAGE:-90}"
if command -v cargo-llvm-cov >/dev/null 2>&1 || cargo llvm-cov --version >/dev/null 2>&1; then
  ./scripts/coverage.sh
  if command -v python3 >/dev/null 2>&1; then
    pct="$(python3 - <<'PY'
import re
total_hit = total = 0
with open("lcov.info") as f:
    for line in f:
        if line.startswith("LF:"):
            total += int(line[3:].strip())
        elif line.startswith("LH:"):
            total_hit += int(line[3:].strip())
if total == 0:
    print(0)
else:
    print(int(100 * total_hit / total))
PY
)"
    echo "llvm-cov line coverage: ${pct}% (min ${MIN_COV}%)"
    if [[ "${pct}" -lt "${MIN_COV}" ]]; then
      echo "quality_gate: coverage below ${MIN_COV}%"
      exit 1
    fi
  else
    echo "quality_gate: python3 not found — skipping coverage percent check"
  fi
else
  echo "quality_gate: cargo-llvm-cov not installed — skipping coverage check"
fi

if [[ "${STOCKVIZ_RUN_MUTANTS:-}" == 1 ]]; then
  if command -v cargo-mutants >/dev/null 2>&1 || cargo mutants --version >/dev/null 2>&1; then
    echo "quality_gate: running cargo mutants (STOCKVIZ_RUN_MUTANTS=1)"
    ./scripts/run_mutants.sh || {
      echo "quality_gate: mutants run failed"
      exit 1
    }
  else
    echo "quality_gate: cargo-mutants not installed — skipping mutants"
  fi
fi

if [[ "${STOCKVIZ_RUN_FUZZ:-}" == 1 ]]; then
  if command -v cargo-fuzz >/dev/null 2>&1; then
    FUZZ_SEC="${STOCKVIZ_FUZZ_SECONDS:-300}"
    echo "quality_gate: running csv_parse fuzz (STOCKVIZ_RUN_FUZZ=1, ${FUZZ_SEC}s)"
    ./scripts/fuzz_csv.sh -max_total_time="${FUZZ_SEC}" || {
      echo "quality_gate: csv_parse fuzz failed"
      exit 1
    }
    echo "quality_gate: running my_target pipeline fuzz (${FUZZ_SEC}s)"
    ./scripts/fuzz_pipeline.sh -max_total_time="${FUZZ_SEC}" || {
      echo "quality_gate: my_target fuzz failed"
      exit 1
    }
  else
    echo "quality_gate: cargo-fuzz not installed — skipping fuzz"
  fi
fi

echo "quality_gate: OK"
