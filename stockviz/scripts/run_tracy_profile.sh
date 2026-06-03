#!/usr/bin/env bash
# r[test.tracing] — Tracy profiler (not tracey). No hard dependencies.
set -euo pipefail
cat <<'EOF'
Tracy (profiler) quick notes — see stock_viz_spec.md r[test.tracing] and §5.

  Note: spec coverage is the **tracey** CLI (with an “e”), e.g. tracey query status —
  not `tracy` (no such stockviz tool; apt may suggest unrelated packages).

  - Build/run the graph binary with your Tracy/tracing capture setup (e.g. tracet)
    and connect the Tracy viewer to the running process.
  - Example (environment varies by Tracy version and setup):
      RUST_LOG=info cargo run -- graph path/to/data.csv

This script intentionally does not install Tracy. See https://github.com/wolfpld/tracy
EOF
