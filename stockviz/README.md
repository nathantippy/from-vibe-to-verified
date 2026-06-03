# StockViz

Minimal Rust demo: download daily OHLCV → CSV → candlestick chart with 50/150 SMA, price labels, and volume.

**Talk materials**

- [Tag appendix & slide map](docs/TALK_TAG_APPENDIX.md)
- [Live runbook](docs/TALK_RUNBOOK.md)
- [Talk-only tracey tags](docs/talk_tags_spec.md)
- [Tracey code-unit mapping guide](docs/TRACEY_CODE_MAPPING.md)
- [Fuzzing guide](docs/FUZZING.md)
- [Mutation testing guide](docs/MUTATION_TESTING.md)
- [Slide deck sync](docs/TALK_SLIDE_SYNC.md)
- [Product spec](stock_viz_spec.md)

**Quick start**

```bash
export TWELVE_DATA_API_KEY=your_key
cargo run -- download AAPL
cargo run -- graph AAPL
```

(`graph` accepts `AAPL` or `AAPL.csv`; extensionless paths get `.csv` appended.)

**CI**

```bash
./scripts/ci_pr_fast.sh              # PR
./scripts/ci_local_default.sh        # full default
STOCKVIZ_TRACEY_STRICT=1 ./scripts/quality_gate.sh   # main gate
```

See [scripts/README.md](scripts/README.md) for the full toolkit.
