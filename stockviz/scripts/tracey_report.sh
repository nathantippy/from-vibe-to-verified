#!/usr/bin/env bash
# r[impl test.tracey.workflow] r[impl repo.scripts]
# Runs eikopf/tracey (v1.x): validate, status, and (strict) uncovered/untested gates.
# Requires `tracey` on PATH and a running workspace daemon (`tracey daemon` or implicit).
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"

if ! command -v tracey >/dev/null 2>&1; then
  echo "tracey: not on PATH — skipping (install: https://github.com/eikopf/tracey ; see scripts/README.md)"
  exit 0
fi

mkdir -p target/tracey
log="${TRACEY_OUTPUT:-target/tracey/validate.log}"
status_log="target/tracey/status.log"
uncovered_log="target/tracey/uncovered.log"
untested_log="target/tracey/untested.log"
unmapped_log="target/tracey/unmapped.log"

trace_tolerant_exit() {
  local rc=$1
  if [[ "$rc" -ne 0 ]]; then
    echo "tracey: command failed (exit $rc) — daemon not running? Try: cd \"$ROOT\" && tracey daemon"
    if [[ "${STOCKVIZ_TRACEY_STRICT:-}" == 1 ]]; then
      exit "$rc"
    fi
    exit 0
  fi
}

# Fail if query output lists requirements (heuristic: lines with r[tag] or "N uncovered" with N>0).
tracey_output_has_gaps() {
  local file=$1
  [[ -f "$file" ]] || return 1
  if rg -q '^r\[[a-z0-9_.]+\]' "$file" 2>/dev/null; then
    return 0
  fi
  if rg -q '[1-9][0-9]* (of [0-9]+ )?(requirements are )?uncovered' "$file" 2>/dev/null; then
    return 0
  fi
  if rg -q '[1-9][0-9]* (of [0-9]+ )?(requirements are )?untested' "$file" 2>/dev/null; then
    return 0
  fi
  if rg -q 'have no implementation reference' "$file" 2>/dev/null \
    && ! rg -q '0 of [0-9]+ requirements are covered' "$file" 2>/dev/null; then
    if rg -q '[1-9][0-9]* of [0-9]+ requirements' "$file" 2>/dev/null; then
      return 0
    fi
  fi
  return 1
}

# Fail when tracey reports unmapped Rust code units (see docs/TRACEY_CODE_MAPPING.md).
tracey_output_has_unmapped() {
  local file=$1
  [[ -f "$file" ]] || return 1
  if rg -q '[1-9][0-9]* unmapped code units' "$file" 2>/dev/null; then
    return 0
  fi
  if rg -q 'unmapped code units out of' "$file" 2>/dev/null \
    && ! rg -q '0 unmapped code units' "$file" 2>/dev/null; then
    return 0
  fi
  return 1
}

set +e
tracey query validate 2>&1 | tee "$log"
val_rc=${PIPESTATUS[0]}
set -e
trace_tolerant_exit "$val_rc"

set +e
json="$(tracey query validate --json 2>/dev/null)"
json_rc=$?
set -e
if [[ "$json_rc" -ne 0 ]] || [[ -z "$json" ]]; then
  echo "tracey: could not read validate --json output"
  if [[ "${STOCKVIZ_TRACEY_STRICT:-}" == 1 ]]; then
    exit 1
  fi
  exit 0
fi

err_count=0
if command -v jq >/dev/null 2>&1; then
  err_count="$(echo "$json" | jq '[.[].errorCount] | add // 0')"
else
  err_count="$(echo "$json" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(sum(x.get("errorCount",0) for x in d))')"
fi

set +e
tracey query status 2>&1 | tee "$status_log"
status_rc=${PIPESTATUS[0]}
set -e
trace_tolerant_exit "$status_rc"

if [[ "${STOCKVIZ_TRACEY_STRICT:-}" == 1 ]]; then
  set +e
  tracey query uncovered 2>&1 | tee "$uncovered_log"
  uncovered_rc=${PIPESTATUS[0]}
  set -e
  trace_tolerant_exit "$uncovered_rc"

  set +e
  tracey query untested 2>&1 | tee "$untested_log"
  untested_rc=${PIPESTATUS[0]}
  set -e
  trace_tolerant_exit "$untested_rc"

  if tracey_output_has_gaps "$uncovered_log"; then
    echo "tracey: uncovered requirements remain — see $uncovered_log"
    exit 1
  fi
  if tracey_output_has_gaps "$untested_log"; then
    echo "tracey: untested requirements remain — see $untested_log"
    exit 1
  fi

  set +e
  tracey query unmapped 2>&1 | tee "$unmapped_log"
  unmapped_rc=${PIPESTATUS[0]}
  set -e
  trace_tolerant_exit "$unmapped_rc"

  if tracey_output_has_unmapped "$unmapped_log"; then
    echo "tracey: unmapped code units remain — see $unmapped_log"
    echo "tracey: map each fn/struct with // r[impl <tag>] (docs/TRACEY_CODE_MAPPING.md)"
    exit 1
  fi

  # Status log: fail when zero coverage reported (e.g. stale daemon or missing impl refs).
  if rg -q '0 of [0-9]+ requirements are covered' "$status_log" 2>/dev/null \
    && rg -q '[1-9][0-9]* have no implementation' "$status_log" 2>/dev/null; then
    echo "tracey: strict mode requires non-zero impl coverage — see $status_log"
    echo "tracey: try: tracey daemon && tracey query status"
    exit 1
  fi
fi

if [[ "${err_count:-0}" -gt 0 ]]; then
  echo "tracey: validate reported $err_count error(s) — see $log"
  exit 1
fi

echo "tracey: validate OK. Logs: $log , $status_log"
if [[ "${STOCKVIZ_TRACEY_STRICT:-}" == 1 ]]; then
  echo "tracey: strict OK (uncovered/untested/unmapped clear). Logs: $uncovered_log , $untested_log , $unmapped_log"
fi
