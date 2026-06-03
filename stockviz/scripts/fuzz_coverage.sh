#!/usr/bin/env bash
# r[impl repo.scripts] r[impl test.fuzz.pipeline]
# Run `cargo fuzz coverage` for my_target and print llvm-cov line summary for core modules.
set -euo pipefail
ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
cd "$ROOT"
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-nightly}"

TARGET="${STOCKVIZ_FUZZ_COVERAGE_TARGET:-my_target}"
echo "fuzz_coverage: generating profdata for ${TARGET} (corpus: fuzz/corpus/${TARGET})"
(cd fuzz && cargo fuzz coverage "${TARGET}")

PROFDATA="$ROOT/fuzz/coverage/${TARGET}/coverage.profdata"
if [[ ! -f "$PROFDATA" ]]; then
  echo "fuzz_coverage: missing ${PROFDATA}" >&2
  exit 1
fi

BIN="$(find "$ROOT/fuzz/target" -path "*/coverage/*/release/${TARGET}" -type f -executable 2>/dev/null | head -1)"
if [[ -z "$BIN" ]]; then
  BIN="$(find "$ROOT/fuzz/target" -path "*/release/${TARGET}" -type f -executable 2>/dev/null | head -1)"
fi
if [[ -z "$BIN" ]]; then
  echo "fuzz_coverage: could not find fuzz binary for ${TARGET} under fuzz/target" >&2
  exit 1
fi

LLVM_COV="$(command -v llvm-cov 2>/dev/null || true)"
if [[ -z "$LLVM_COV" ]]; then
  SYSROOT="$(rustc +nightly --print sysroot 2>/dev/null || true)"
  for cand in \
    "${SYSROOT}/lib/rustlib/$(rustc +nightly -vV 2>/dev/null | sed -n 's/^host: //p')/bin/llvm-cov" \
    "${SYSROOT}/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov"; do
    if [[ -n "$cand" && -x "$cand" ]]; then
      LLVM_COV="$cand"
      break
    fi
  done
fi
if [[ -z "$LLVM_COV" ]]; then
  echo "fuzz_coverage: llvm-cov not found (rustup component add llvm-tools-preview)" >&2
  echo "fuzz_coverage: profdata at ${PROFDATA}" >&2
  exit 1
fi

echo "fuzz_coverage: binary ${BIN}"
echo "fuzz_coverage: summary (data, chart, fuzz_harness, download)"
"$LLVM_COV" report "$BIN" \
  -instr-profile="$PROFDATA" \
  -ignore-filename-regex='(fuzz/|gui/|tests/)' \
  "$ROOT/src/data.rs" \
  "$ROOT/src/chart.rs" \
  "$ROOT/src/fuzz_harness.rs" \
  "$ROOT/src/download/twelve_data.rs" 2>/dev/null || {
  echo "fuzz_coverage: llvm-cov report failed; trying show (first 80 lines)"
  "$LLVM_COV" show "$BIN" \
    -instr-profile="$PROFDATA" \
    -ignore-filename-regex='(fuzz/|gui/)' \
    "$ROOT/src/data.rs" \
    "$ROOT/src/chart.rs" \
    "$ROOT/src/fuzz_harness.rs" | head -80
}

HTML="${STOCKVIZ_FUZZ_COVERAGE_HTML:-$ROOT/target/fuzz_coverage_${TARGET}.html}"
if [[ "${STOCKVIZ_FUZZ_COVERAGE_HTML:-}" != "0" ]]; then
  "$LLVM_COV" show "$BIN" \
    -instr-profile="$PROFDATA" \
    -ignore-filename-regex='(fuzz/|gui/|tests/)' \
    -format=html \
    -output-dir="$HTML" \
    "$ROOT/src" 2>/dev/null && echo "fuzz_coverage: HTML → ${HTML}/index.html" || true
fi

echo "fuzz_coverage: done"
