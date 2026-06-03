# Tracey code unit mapping

StockViz uses tracey for **two separate scores**. Both matter for teaching spec-driven development.

## Two metrics

| Metric | Command | What it means |
|--------|---------|---------------|
| **Requirement coverage** | `tracey query status` | Every `r[tag]` in the spec has at least one `r[impl]` and `r[verify]` somewhere in the repo. |
| **Code unit mapping** | `tracey query unmapped` | Every Rust **code unit** (function, struct, enum, impl method, module) in the tracey scan path is linked to at least one `r[impl]`. |

You can have **65/65 requirements covered** and still show unmapped code units if annotations sit only in file headers.

## Before vs after (anti-pattern)

**Wrong** — tags only in the module doc; tracey still reports `gui/app.rs` at 0%:

```rust
//! Main chart window
// r[impl gui.chart.volume]
// r[impl gui.chart.zoom]

pub struct StockvizApp { ... }

fn paint_volume(...) { ... }  // unmapped
```

**Right** — tag on the unit that does the work:

```rust
// r[impl gui.chart.volume]
fn paint_volume(&self, painter: &egui::Painter, rect: Rect, slice: &[Bar]) {
    ...
}
```

## Mapping rules

1. Put `// r[impl <tag>]` on the **code unit itself** (line above `fn` / `struct` / `pub mod`).
2. Use **one primary tag** — the requirement that best explains why this unit exists.
3. Add a **second tag** only when the unit clearly implements two requirements (e.g. `parse_csv_str` → `data.format` + `data.validation`).
4. **Private helpers** get the same tag as the feature they support.
5. **`r[verify]`** belongs on tests; production code uses **`r[impl]`**.
6. Tags must exist in [`stock_viz_spec.md`](../stock_viz_spec.md) or [`talk_tags_spec.md`](talk_tags_spec.md).

## Grep recipes (student lab)

```bash
# Where is gui.chart.volume implemented?
rg 'r\[impl gui\.chart\.volume\]' -n src/

# All impl sites for a tag
rg 'r\[impl cli\.download\]' .

# Requirement text in specs
rg 'r\[cli\.download\]' stock_viz_spec.md docs/talk_tags_spec.md
```

Expected for `gui.chart.volume`:

- `src/gui/app.rs` — `paint_volume`
- `src/chart.rs` — `volume_sum`, `buckets_for_width` (dual-tagged with candles)

## Anti-patterns

| Anti-pattern | Why it fails |
|--------------|--------------|
| File-level tags only | Children stay unmapped |
| Tag `main` with every requirement | Unreadable; use `app.overview` / `app.errors` |
| Invent tags not in spec | `tracey query validate` errors |
| `r[verify]` on production `fn` | verify is for tests |
| Excluding `src/**` from styx to go green | Defeats the lesson |

## Presenter checks

```bash
tracey daemon
tracey query status      # 65 of 65 covered
tracey query unmapped      # 0 unmapped code units
STOCKVIZ_TRACEY_STRICT=1 ./scripts/tracey_report.sh
```

## Live demo: unmapped vs uncovered

| Break | Command | Lesson |
|-------|---------|--------|
| Comment out `// r[impl gui.chart.zoom]` on `zoom_in` | `tracey query unmapped` | Code without declared intent |
| Remove `// r[verify gui.chart.zoom]` on a test | `tracey query untested` | Requirement not proven |

Restore both before committing.

## Slop-catch script (30 seconds)

1. Comment out `// r[impl gui.chart.zoom]` above `zoom_in` in `src/chart.rs`.
2. Run `tracey query unmapped --path src/chart.rs`.
3. Restore the line; confirm 0 unmapped.

See also [`TALK_RUNBOOK.md`](TALK_RUNBOOK.md) and [`TALK_TAG_APPENDIX.md`](TALK_TAG_APPENDIX.md).
