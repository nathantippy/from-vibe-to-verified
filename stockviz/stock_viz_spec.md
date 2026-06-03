**StockViz – Simple Demo Project**  
**For the “AI Agents + Rust Spec-Driven Testing” Talk (2026)**

**Project Goal**  
A **minimal, easy-to-understand** Rust app that everyone can follow in 5 minutes.  
Downloads daily stock data → saves to CSV → displays **candlesticks**, **50- and 150-day moving averages**, and a **volume histogram** (typical stacked chart layout).

r[app.identity]

Talk binary and environment: **`stockviz`**, Ubuntu native, pinned egui/eframe 0.34 (aligns front matter with tracey-visible identity rules).
- **Binary name:** `stockviz`
- **Platform:** Ubuntu Linux (native)
- **Pinned / named crates in this spec:** `egui = "0.34"`, `eframe = "0.34"`, `egui_kittest`, `log`, `flexi_logger` (see §5 Logging). **Additional crates:** §8 Dependencies (informative); versions follow “latest compatible at talk date” unless pinned in `Cargo.lock`.
- **Requirement tag style (markdown):** `r[category.descriptive.name]` (idiomatic dot-separated; see [eikopf/tracey](https://github.com/eikopf/tracey)).

**Table of contents**  
§0 Build-Time Data Provider · §1 What the App Does · §2 Data & CLI · §3 Chart Rendering · §4 Errors & Process Exit · §5 Logging · §6 Testing & Traceability · §7 Non-Functional · §8 Dependencies

---

### 0. Build-Time Data Provider

r[build.provider]

Data access is **compile-time only**: exactly **one** market-data backend is linked into the binary. There is **no runtime provider switch**.

**Cargo features**
- **`twelve-data`**: Twelve Data API (enabled by **default**).
- **`schwab`**: Charles Schwab API (opt-in).

**Default build (Twelve Data)**
```bash
cargo build
```

**Schwab build**
```bash
cargo build --no-default-features --features schwab
```

r[build.provider.exclusive]

Enabling **both** `twelve-data` and `schwab` at once is invalid. The implementation **must** fail at compile time (e.g. `compile_error!` under conflicting `cfg`).

**Credentials & CLI by provider**

r[cli.download.twelvedata]

Compiled only with `twelve-data`.
- API key: `--api-key` **or** environment variable `TWELVE_DATA_API_KEY`.

r[cli.download.twelvedata.api]

Twelve Data download semantics.
- Use Twelve Data **daily** OHLCV time series; request the **maximum history** the chosen endpoint allows for the symbol. **Demo implementation:** `outputsize=5000` (API cap per request; see `src/download/twelve_data.rs`).
- Use the **split-adjusted daily series** (adjusted OHLC/close as provided by Twelve Data for continuous historical charts). Document the exact HTTP path and parameters in source comments next to the client.
- **Dates:** normalize the API’s date field to **date-only** `YYYY-MM-DD` in the **`America/New_York` calendar** (strip time-of-day; if the API returns a timestamp with offset, convert to NY calendar date; **reject** the download if a row cannot be mapped unambiguously).
- **Symbol:** normalize the CLI ticker to **uppercase ASCII** before the request. HTTP/API failure (4xx/5xx, invalid body) → `Result::Err` and top-level **`error!` + exit 1`**, per §4.
- **Response size (download path):** after a successful HTTP response, the CLI download parser **must** accept JSON bodies up to **`MAX_DOWNLOAD_JSON_BODY`** (1 MiB, `src/download/twelve_data.rs`) and up to **5000** `values` rows (`MAX_TIME_SERIES_ROWS` in the same file).
- **Response size (pipeline/fuzz only):** the **64 KiB** cap **`MAX_PIPELINE_PAYLOAD`** (`src/test_inputs.rs`) **must not** apply to CLI download; it **only** applies to pipeline/fuzz JSON parsing per **`r[test.fuzz.pipeline]`**.

r[cli.download.schwab]

Compiled only with `schwab` — bounded acceptance (stub allowed for talk v1).
- **Talk v1 scope:** the `schwab` feature **may** ship as a **compile-success stub** that returns a single clear error on `download` (`unimplemented!`-class messaging via `log::error!` + non-zero exit) **until** the checklist below is completed. Alternatively, implement full parity with Twelve Data when credentials exist.
- **When implemented, must match:** same **§2** CSV contract, same **`download` / `graph` UX**, and same **§3** chart behavior as the `twelve-data` build (only auth and upstream API differ).
- **Placeholder configuration surface** (exact names **TBD** until Schwab work lands): e.g. `SCHWAB_API_KEY`, `SCHWAB_ACCESS_TOKEN`, `SCHWAB_REFRESH_TOKEN`, optional `SCHWAB_ACCOUNT_ID`, and paths for on-disk OAuth material—**replace placeholders** during implementation.
- **Checklist before removing “stub allowed”:** (1) Link Schwab’s **official** authentication / market-data documentation in this spec (replace this bullet with the URL). (2) List **final** env vars and files. (3) Document token refresh and any required CLI flags.

`stockviz download <SYMBOL>` behavior is otherwise identical: maximum available daily OHLCV → `<SYMBOL>.csv` in the current directory. See §2 (**`r[cli.commands]`**, **`r[data.format]`**) and §5 Logging for operational output.

---

### 1. What the App Does

r[app.overview]

CLI download and graph subcommands plus maximized two-pane GUI.
- **CLI mode** (exact syntax: **`r[cli.commands]`**)  
  `stockviz download AAPL` → fetches max daily history from the **compile-time-selected** provider and writes `AAPL.csv` (see **§0**).  
  `stockviz graph AAPL` or `stockviz graph AAPL.csv` → launches GUI (see **`r[cli.graph.path]`**)

r[gui.core]

- **GUI mode** — maximized window on launch (desktop size).  
  - **Two-pane chart (stacked vertically):** **price pane** — candlesticks + 50-day SMA (**`r[data.sma]`**) + 150-day SMA (**`r[data.sma150]`**) + compact SMA legend (**`r[gui.chart.sma.legend]`**) + right-edge price labels (**`r[gui.chart.yticks]`**); **volume pane** — volume histogram (**`r[gui.chart.volume]`**).  
  - Controls: OS close button + two buttons: **Zoom In (+)** and **Zoom Out (-)**  
  - No menus, no side panels, no sliders, no panning, no general legends (SMA-only legend per **`r[gui.chart.sma.legend]`**)

---

### 2. Data & CLI

r[cli]

Data and CLI surface: commands, CSV contract, validation, and SMA.

r[cli.commands]

Exact user-visible interface.
- **Subcommands**
  - `stockviz download <SYMBOL>` — fetches data; writes `<SYMBOL>.csv` in the process **current working directory** (overwrite).
  - `stockviz graph <SYMBOL_OR_PATH>` — loads daily OHLCV CSV; relative paths resolve against **CWD** (see **`r[cli.graph.path]`**).
- **Flags**
  - **`twelve-data` builds:** optional `--api-key <KEY>` (overrides `TWELVE_DATA_API_KEY`). No other download flags required for the demo.
  - **`schwab` builds:** no `--api-key`; authentication per **§0 / `r[cli.download.schwab]`** (env / future flags).
  - Standard: **`-h` / `--help`**, **`-V` / `--version`** (`clap`-style) on the root command.
- **Exit codes (all builds):** **`0`** success; **`1`** any fatal error (see §4).

r[cli.download]

- Downloads **maximum available daily OHLCV** bars  
- Saves to `<SYMBOL>.csv` (overwrites) in current folder  
- Provider-specific authentication and HTTP semantics: **§0** (`r[cli.download.twelvedata]`, **`r[cli.download.twelvedata.api]`**, `r[cli.download.schwab]`).  
- Operational logging uses `log` macros with **`flexi_logger`** initialization (see §5 Logging).

r[cli.graph.path]

Graph CSV path resolution.
- If the CLI argument has **no file extension**, resolve by appending **`.csv`** (e.g. `NOW` → `NOW.csv`, `reports/Q1` → `reports/Q1.csv`), relative to **CWD**.
- If the argument already has extension **`.csv`**, use the path unchanged.
- Any **other** extension **must** be rejected with a clear error (do not load non-CSV files).
- Missing file or **`r[data.validation]`** failure → §4 (`error!`, **`exit(1)`**).

r[data.format]

CSV header (exactly):
```
Date,Open,High,Low,Close,Volume
```
- Oldest → newest  
- `Date` = `YYYY-MM-DD` (calendar date in **`America/New_York`**; daily bars only, see §3)  
- All numbers = `f64`

r[data.validation]

Required before graph / SMA.
After parsing the CSV for `graph`:
- **Ordering:** `Date` values are **strictly ascending**; **no duplicate** dates; **reject** unsorted or duplicate rows.
- **Numeric sanity:** all OHLCV fields are **finite** `f64` (**reject** NaN / ±inf).
- **Volume:** **`Volume >= 0.0`** (**reject** negatives); **zero** volume is allowed.
- **OHLC consistency:** `High >= max(Open, Close)` and `Low <= min(Open, Close)` for every row; **reject** inconsistent rows (no clamping in this demo).
- On any violation: **§4** (`error!`, **`exit(1)`**) for the `graph` command.

r[data.sma]

50-period simple moving average.
- Computed on **`Close`** only.
- **Definition:** rolling arithmetic mean over **exactly the previous 50 rows** (bars) in chronological order. At zero-based row index *i*, the SMA value exists **iff** `i >= 49`, using rows `i-49` … *i* inclusive.
- This is **not** a 50-**calendar**-day window with synthetic weekends; it is **50 CSV rows** = **50 trading days as stored**.
- Plot the SMA polyline only where values exist (**≥50 rows** in the series). Aligns with **`r[gui.chart.candles]`** SMA display.

r[data.sma150]

150-period simple moving average.
- Computed on **`Close`** only.
- **Definition:** rolling arithmetic mean over **exactly the previous 150 rows** (bars) in chronological order. At zero-based row index *i*, the SMA value exists **iff** `i >= 149`, using rows `i-149` … *i* inclusive.
- This is **not** a 150-**calendar**-day window with synthetic weekends; it is **150 CSV rows** = **150 trading days as stored**.
- Plot the SMA polyline only where values exist (**≥150 rows** in the series). Aligns with **`r[gui.chart.candles]`** SMA display.

r[gui.chart.sma.align]

SMA horizontal alignment with compressed candles.
- Each SMA polyline point for visible bar index *i* in the price slice **must** use the **same horizontal bucket** as the candle for that bar: X = center of the bucket produced by **`buckets_for_pane(slice, width_px)`** (same partition as **`r[gui.chart.candles]`**). When multiple bars share one bucket, they **may** share the same X.
- SMA **Y** values **must** come from **`r[data.sma]`** or **`r[data.sma150]`** respectively; this rule governs **X** placement only.

r[gui.chart.width.fill]

Horizontal pane fill for visible series.
- **`pane_width_px`** means the **shared drawable chart width** per **`r[gui.chart.pane.align]`** (full pane widget width minus the price Y-label margin when **`r[gui.chart.yticks]`** is active)—not the raw widget width when that margin is reserved.
- Price and volume panes **must** lay out with column count **`cols = min(visible_bar_count, pane_width_px)`** (4096 cap), not one column per pixel when that would leave empty trailing columns.
- Non-empty buckets **must** span **`[chart_left, chart_right]`** of the shared drawable rect with equal column width; the last bar in the visible slice (right anchor per **`r[gui.chart.anchor]`**) **must** map to the **last** layout column.

r[gui.chart.pane.align]

Cross-pane horizontal chart geometry (price + volume).
- **Shared drawable rect** for **both** stacked panes:
  - **`chart_left`** = pane left edge.
  - **`chart_width_px`** = full pane width minus **`PRICE_LABEL_MARGIN_PX`** (~48 px per **`r[gui.chart.yticks]`**), minimum 1 px.
- **Price pane:** candles, SMAs, and date X ticks use this drawable rect; Y price labels render in the reserved right margin only.
- **Volume pane:** histogram bars **must** use the **same** **`chart_left`** and **`chart_width_px`** for bucketing and bar X placement; the right margin strip stays **empty** (no volume bars).
- **Column parity:** for the same visible bar slice, bucket count and per-bucket horizontal centers **must** match between price and volume (pairs with **`r[gui.chart.sma.align]`** and bucket partition rules).

---

### 3. Chart Rendering

r[gui.chart]

Chart rendering: candles, volume, anchor, zoom, resize, and time axis.

r[gui.chart.candles]

Price pane.
- Candle coloring (**common retail convention**): **bullish (green)** when `Close >= Open`; **bearish (red)** when `Close < Open`.  
- Wick color matches the candle body (same green/red as the body).  
- **Stable demo colors (sRGB approximate, for kittest / screenshots):** bull body/wick **#26a69a**; bear body/wick **#ef5350**; SMA 50 line **#42a5f5**; SMA 150 line **#ffa726**. (If `egui` defaults drift, tests should assert these or document `Color32::from_rgb` equivalents in code comments.)
- 50-period Simple Moving Average on **`Close`**, per **`r[data.sma]`** (blue **`#42a5f5`**).
- 150-period Simple Moving Average on **`Close`**, per **`r[data.sma150]`** (orange **`#ffa726`**).
- Y-axis: auto price scale from visible OHLC **and both SMA series**; sparse numeric labels on the right edge per **`r[gui.chart.yticks]`**.
- Compact SMA legend per **`r[gui.chart.sma.legend]`** (top-left overlay in the price pane).
- If window is narrow → multiple trading days **must** compress into one horizontal bucket (grouped candle): **high** = max of highs, **low** = min of lows, **open** = open of the **first** day in the bucket (chronological), **close** = close of the **last** day in the bucket. **Volume in compressed buckets:** **`sum`** per **`r[gui.chart.volume]`** (do not duplicate aggregation rules here).
- At **full-history zoom** (visible span = all bars through **`D_anchor`**), the painted slice **must** include **every** loaded row; compression **must** be via bucketing only—not by dropping older bars from the time slice.

r[gui.chart.sma.legend]

Compact SMA identification legend.
- **Location:** top-left overlay inside the **price pane** (not a side panel).
- **Entries:** colored short line sample + text **`SMA 50`** and **`SMA 150`**, using the same colors as the plotted lines (**50:** **`#42a5f5`**; **150:** **`#ffa726`**).
- No other legend entries (candles and volume stay unlabeled).

r[gui.chart.yticks]

Sparse price labels on the right edge of the price pane.
- **Location:** inside the **price pane** on the **right edge** (not a separate panel).
- **Values:** derived from the same auto Y range as candles and both SMAs (visible slice, 5% padding).
- **Density:** tick count from available **height**, mirroring the sparse philosophy of **`r[gui.chart.xticks]`**; use “nice” rounded intervals (implementation-defined heuristic, covered by tests).
- **Layout:** reserve a fixed right margin of **~48 px** for label text so candles are not clipped.
- **Scope:** price pane only (volume Y labels remain optional per **`r[gui.chart.volume]`**).

r[gui.chart.volume]

Volume pane.
- **Layout:** **Stacked panes**—**price pane** (**`r[gui.chart.candles]`** + SMA) **above**; **volume pane** (**histogram**) **below**. No side or third panels.
- **Height split:** allocate **~78%** of the **client chart area** to the price pane and **~22%** to the volume pane, with a **minimum volume strip height of 96 px**. If the window is too short to satisfy defaults, **shrink the price pane first** until the volume strip is at least **96 px** (never drop the volume pane entirely).
- **Horizontal coupling:** Volume shares the **same visible time range**, **`D_anchor`**, **zoom level**, **resize behavior**, **shared drawable chart geometry** (**`r[gui.chart.pane.align]`**, including the Y-label margin reserved by **`r[gui.chart.yticks]`**), and **horizontal pixel-column bucketing** as the price pane (**`r[gui.chart.anchor]`**, **`r[gui.chart.zoom]`**, **`r[gui.chart.resize]`**, **`r[gui.chart.xticks]`**). **Vertical** scaling is independent (price auto-scales on OHLC/SMA; volume auto-scales on volume).
- **Bars:** One vertical bar per **visible bucket** (one per day when not compressed). **Width** matches candle/body horizontal spacing; baseline at the **bottom** of the volume pane.
- **Colors:** Match the **paired candle** for that bucket: bull **#26a69a**, bear **#ef5350** (same **`Close >= Open`** / **`<`** rule as price).
- **Y scale:** Auto-scale from **0** through **max summed volume** in the **visible** window (post-bucketing). **Y-axis numeric labels optional**; if shown, they **must not clip** bars.
- **Aggregation:** When multiple days map to one horizontal bucket, **`sum`** `Volume` across rows in that bucket (normative; pairs with OHLC aggregation in **`r[gui.chart.candles]`**).

r[gui.chart.timeaxis]

- Data is **daily OHLCV** only; **no intraday timestamps** and **no live “market clock”** in the UI.  
- **Trading calendar semantics** use timezone **`America/New_York`**. CSV `Date` values are interpreted as NY **calendar** dates (no time-of-day).

r[gui.chart.anchor]

Right-edge anchor date.
- Let `D_cal` = current **calendar date** in **`America/New_York`** (evaluated when rendering / loading).
- Let `D_last` = the last **`Date`** in the loaded CSV **after** **`r[data.validation]`**.
- **Chart anchor date** `D_anchor = min(D_cal, D_last)` (by chronological date order).
- The visible window’s **right edge** aligns to **`D_anchor`** for **both** price and volume panes. Do **not** leave empty “future” chart space to the right of the final bar when EOD data ends before `D_cal` (weekends, holidays, partial history).

r[gui.chart.xticks]

- Date labels on the X-axis are **sparse**—avoid labeling every bar.  
- Choose tick granularity (**week, month, quarter**, or coarser) from **available horizontal space** and **zoom level**, preferring **nice calendar boundaries** so labels stay readable.  
- Exact heuristic is implementation-defined but must be covered by tests tagged to this requirement.

r[gui.chart.zoom]

- Right edge **always locked** to **`D_anchor`** from **`r[gui.chart.anchor]`** (user cannot move the anchor).  
- Zoom In (`+`): halves the visible time span (more detail), **subject to `r[gui.chart.zoom.limits]`**  
- Zoom Out (`-`): doubles the visible time span (up to full history)  
- Starts at full history on load  
- **Scope:** applies to **both** panes (same visible X range).

r[gui.chart.zoom.limits]

Zoom extent.
- **Minimum** visible span: **7** trading rows (daily bars in the series); **Zoom In (`+`)** must **not** narrow past this floor (if the series has fewer than 7 rows, the floor is **`max_bars`**). Constant: **`MIN_ZOOM_VISIBLE_BARS`** in `src/chart.rs`.  
- **Maximum** visible span: **full** loaded history (same as the “zoom out” cap).

r[gui.chart.resize]

Window resize behavior (horizontal scope locked across panes).

- **Horizontal scope:** **`r[gui.chart.resize]`** applies to **both** panes; **horizontal** time extent stays **locked together** (**no** independent vertical-only resize that breaks alignment). **Vertical** split still follows **`r[gui.chart.volume]`** (price vs volume strip).
- **Full-history zoom** (`visible_bars` = full span through anchor): widening or narrowing the window **must not** change the time span (always all bars to anchor); only bucket width / days-per-column changes.  
- **Partial zoom** (user has zoomed in):  
  - When the window is **made larger** (wider): keep the **same zoom** and show **more history** on the left (right edge anchored). **Only** if there is not enough history left, automatically zoom out toward full dataset.  
  - When the window is **made smaller** (narrower): keep the **same zoom** and crop from the left (right edge anchored).  

- **Pixel density:** Window resize **never** breaks **horizontal** bucket alignment between panes; at full-history zoom, resize changes compression only. (Each pane’s **vertical** autoscale updates normally.)

---

### 4. Errors & Process Exit

r[app.errors]

Result-based errors, thiserror, log::error on fatal exit, code 1.
- **Internal style:** use idiomatic Rust **`Result<T, E>`**; propagate errors with `?`. Prefer **`thiserror`** for a project **`Error`** enum with a clear **`Display`** (and **`source()`** / error chain where applicable); avoid **`anyhow`** in library-shaped modules so error types stay explicit.  
- **Binary entry (`main`):** on any fatal error path, emit **`error!(...)`** with **full diagnostic context** (at minimum **`Display` and `{:?}` / structured fields** so operators see the complete failure), then **`std::process::exit(1)`**.  
- **CLI:** all hard failures: **`error!` then `exit`** as above.  
- **GUI launch (`graph`):** unrecoverable errors (e.g. cannot read CSV, **`r[data.validation]`** failure) **log via `error!` and exit with code 1**—no modal dialogs required for this demo.

---

### 5. Logging

r[app.logging]

flexi_logger + log macros; RUST_LOG-style filter.

r[app.logging.tracing]

tracing spans for Tracy profiler (complements log, does not replace).
- Initialize **`flexi_logger`** early in `main` (before substantive work).  
- Use the **`log`** crate macros (`error!`, `warn!`, `info!`, `trace!`, etc.) for human-readable application logs.  
- **Filter / verbosity** follow **`flexi_logger`** defaults compatible with a standard **`RUST_LOG`**-style environment override (document the exact env var in code comments if the crate’s default differs).  

---

### 6. Testing & Traceability – The Full 2026 Stack

r[test.strategy]

Full 2026 testing stack: nextest, tracey, llvm-cov, proptest, mutants, fuzz, bacon, kittest.
This project is deliberately simple so we can clearly show **every tool**

| Tool          | How we use it in StockViz (demo-ready)                              | Tracey tag example                  |
|---------------|---------------------------------------------------------------------|-------------------------------------|
| **tracey**    | Every requirement in this spec is tagged. Code has `// r[impl ...]` and `// r[verify ...]` comments. Full coverage report in CI. | `r[gui.chart.resize]`              |
| **tracey (code units)** | Every Rust code unit under the tracey `impls` include path (`src/**/*.rs`, fuzz targets) must declare `// r[impl <tag>]` on the unit itself (not only file headers). `tracey query unmapped` must report zero in strict CI. See `docs/TRACEY_CODE_MAPPING.md`. | `r[test.tracey.coverage]` |
| **cargo test / nextest** | All unit + integration tests run with `nextest` (fast parallel). Provider-specific integration tests run the relevant crate features (`twelve-data` vs `schwab`). | `r[build.provider]`              |
| **egui_kittest** | Automated UI tests: window resize, **zoom +/−**, **volume pane** alignment/coloring, candle rendering | `r[test.kittest.resize]`           |
| **llvm-cov**  | Line + branch coverage on all code (shown live in talk); main branch **`scripts/quality_gate.sh`** enforces **≥90%** line coverage (`STOCKVIZ_MIN_COVERAGE`, default 90). | `r[talk.llvm.cov]`                  |
| **proptest**  | Property-based tests via shared **`Arbitrary`** types in `src/test_inputs.rs` (`proptest-arbitrary-interop`) | `r[test.proptest.sma]`, `r[test.proptest.zoom.limits]`, `r[test.arbitrary.shared]` |
| **arbitrary** | Single generator module for structured proptest + `my_target` fuzz (`PipelineFuzzInput`, layout/SMA series types) | `r[test.arbitrary.shared]`, `r[test.arbitrary.proptest]` |
| **mutants**   | Mutation testing on chart logic and data parsing (see `docs/MUTATION_TESTING.md`) | `r[talk.mutants]`                   |
| **fuzz**      | CSV parser + data/chart pipeline (malformed input, round-trip, JSON) | `r[test.fuzz.csv]`, `r[test.fuzz.pipeline]` |
| **bacon**     | Runs in background on save – instant feedback while coding        | `r[talk.bacon]`                     |

r[test.kittest.resize]

Normative egui_kittest zoom and resize coverage.
- **`egui_kittest`** tests **must** exercise **`+` / `−`** zoom and **window resize** behavior against **`r[gui.chart.zoom]`**, **`r[gui.chart.zoom.limits]`**, and **`r[gui.chart.resize]`** (including **shared horizontal** range for price + volume).

r[test.kittest.volume]

Normative volume pane alignment and bull/bear colors.
- Tests **must** assert the **volume pane** is present, **horizontally X-aligned** with the price pane per **`r[gui.chart.pane.align]`** (same drawable width and bucket centers), and that **volume bar colors** follow the same **bull/bear** rule as candles for representative fixtures.

r[test.arbitrary.shared]

Normative shared structured generators for proptest and pipeline fuzz.
- All **`Arbitrary`** types for chart layout, valid bar series, SMA close vectors, and **`PipelineFuzzInput`** **must** live in **`src/test_inputs.rs`** (not duplicated in proptest or fuzz modules).
- **`csv_parse`** remains unstructured **`&[u8]`**; structured types are for **`my_target`** and proptest only.

r[test.arbitrary.proptest]

Normative proptest use of shared **`Arbitrary`** types.
- Every **`r[test.proptest.*]`** property **must** take input via **`proptest_arbitrary_interop::arb::<T>()`** where **`T`** is defined in **`src/test_inputs.rs`**.

r[test.proptest.sma]

Normative 50-period SMA and CSV round-trip properties.
- Properties **must** cover: **50-period** SMA equals rolling mean of **`Close`** over **50 rows** with indices **`i >= 49`** per **`r[data.sma]`**; **monotonic** valid CSV round-trips (parse → canonical serialize) do not drift **`Date`** ordering or row count.
- Inputs **must** use **`Sma50Series`** and **`CsvRoundtripSeries`** from **`r[test.arbitrary.shared]`**.

r[test.proptest.sma150]

Normative 150-period SMA properties.
- Properties **must** cover: 150-period SMA equals rolling mean of **`Close`** over **150 rows** with indices **`i >= 149`** per **`r[data.sma150]`**.

r[test.proptest.chart]

Normative chart bucket partition properties.
- Properties **must** assert: for non-empty bar series and positive width, **`buckets_for_width`** returns **`width.min(4096)`** buckets and the **sum of row counts** equals **`bars.len()`** per **`r[gui.chart.candles]`**.

r[test.proptest.sma.align]

Normative SMA/candle horizontal alignment properties.
- Properties **must** assert: for random bar counts and bucket widths, **`bucket_index_for_bar`** matches the partition implied by **`buckets_for_pane`**, and per-bar SMA screen X equals **`bucket_center_x`** for that bucket (per **`r[gui.chart.sma.align]`**).

r[test.proptest.chart.width.fill]

Normative full-width pane layout properties.
- Properties **must** assert: **`layout_column_count(n, width)`** equals **`min(n, width.min(4096))`** (≥ 1 when `n > 0`), and **`last_bar_center_x`** for the last bar reaches the right portion of the pane (per **`r[gui.chart.width.fill]`**; **`width`** is the shared drawable chart width per **`r[gui.chart.pane.align]`**).

r[test.proptest.pane.align]

Normative cross-pane horizontal alignment properties.
- Properties **must** assert: for random bar counts and full pane widths, price and volume layouts using **`pane_chart_width_px`** produce **identical** bucket counts and **`bucket_center_x`** for every bucket index (per **`r[gui.chart.pane.align]`**).

r[test.proptest.zoom.limits]

Normative zoom-in floor properties.
- Properties **must** use **`ZoomSpanParams`** from **`r[test.arbitrary.shared]`** via **`proptest_arbitrary_interop::arb::<T>()`**.
- Repeated **`zoom_in`** on **`visible_bars`** must never yield a value **below `MIN_ZOOM_VISIBLE_BARS` (7)** when **`max_bars >= 7`**; when **`max_bars < 7`**, the floor is **`max_bars`**.

r[test.proptest.download.json]

Normative Twelve Data JSON dual byte-limit properties.
- Properties **must** use **`TimeSeriesJsonBody`** from **`r[test.arbitrary.shared]`** via **`proptest_arbitrary_interop::arb::<T>()`**.
- For valid Twelve Data–shaped JSON built from ascending valid bars:
  - If **`body.len() <= MAX_PIPELINE_PAYLOAD`** and **`rows <= 5000`**, pipeline (`parse_time_series_json`) and download (`parse_time_series_json_for_download`) parsers **must** agree (both **`Ok`** with the same bar count).
  - If **`MAX_PIPELINE_PAYLOAD < body.len() <= MAX_DOWNLOAD_JSON_BODY`**, pipeline **must** reject; download **must** accept.
  - If **`body.len() > MAX_DOWNLOAD_JSON_BODY`**, download **must** reject.
- Row count **`> 5000`** **must** reject on both paths.

r[test.fuzz.csv]

Normative CSV parser fuzz harness (safe rejection, no panic).
- Fuzz targets **must** include: malformed / wrong headers, **non-finite** numeric fields, **negative volume**, **unsorted** or **duplicate** dates, and **oversized** files (stress), expecting **safe rejection** (no panic) consistent with **`r[data.validation]`**.

r[test.fuzz.pipeline]

Normative pipeline fuzz harness (`my_target`: CSV and/or Twelve Data JSON → chart/SMA math).
- **`my_target`** **must** decode **`PipelineFuzzInput`** from fuzz bytes using **`from_legacy_bytes`** / **`Arbitrary`** (see **`r[test.arbitrary.shared]`**).
- Wire format for corpus seeds: **`[mode u8][payload][64-byte ChartFuzzCtrl]`** via **`PipelineFuzzInput::encode_legacy`** / **`from_legacy_bytes`**.
- **`Result::Err`** on bad data is allowed; **panic** is not.
- Pipeline payload **must** be bounded to **`MAX_PIPELINE_PAYLOAD`** (65 KiB) on decode; parsed series **must** not exceed **5000** rows (Twelve Data cap).
- JSON parsing in the fuzz harness **must** use the **pipeline** parser (`parse_time_series_json`, 64 KiB cap), **not** the CLI download limit (`MAX_DOWNLOAD_JSON_BODY`).
- On successful parse, harness **must** exercise chart bucketing, SMA, zoom/anchor helpers, and CSV **write → re-parse** round-trip within bounded row counts.

r[repo.scripts]

Repository bash entrypoints under scripts/.
- The repository **must** ship **executable** **`bash`** scripts under **`scripts/`**, each using **`set -euo pipefail`** and changing to the repo root (`git rev-parse --show-toplevel` or equivalent) so they work from any working directory.
- **Tracy** vs **tracey:** **Tracy** is the **CPU profiler** used with **`tracing`** spans (**`r[test.tracing]`**, §5). **tracey** is the **spec-tag coverage** tool (**`r[test.tracey.workflow]`**, **`r[test.tracey.coverage]`**). Do not conflate them in docs or script names.
- **CI alignment:** Continuous integration **should** invoke these scripts (or the exact same commands they wrap) so local runs and CI stay identical. See [`.github/workflows/ci.yml`](.github/workflows/ci.yml).

| Script | Purpose | Prerequisite notes |
|--------|---------|---------------------|
| **`scripts/check_fmt.sh`** | `cargo fmt --all -- --check` | `rustfmt` component |
| **`scripts/clippy_default.sh`** | Clippy, default features | `clippy` component |
| **`scripts/clippy_schwab.sh`** | Clippy, `schwab` only | same |
| **`scripts/nextest_default.sh`** | `cargo nextest run` (default) | `cargo-nextest` |
| **`scripts/nextest_schwab.sh`** | `nextest` with `--no-default-features --features schwab` | same |
| **`scripts/coverage.sh`** | `cargo llvm-cov nextest --lcov --output-path lcov.info` | `llvm-tools-preview`, `cargo-llvm-cov` |
| **`scripts/verify_provider_exclusive.sh`** | Assert **`twelve-data` + `schwab`** fails compile (**`r[build.provider.exclusive]`)** | stable `cargo` |
| **`scripts/tracey_report.sh`** | Runs **`tracey query validate`** and **`tracey query status`** (eikopf **tracey** 1.x); writes **`target/tracey/validate.log`** and **`target/tracey/status.log`**. Missing **`tracey`** → **exit 0**. Daemon / **`query`** failures → **exit 0** unless **`STOCKVIZ_TRACEY_STRICT=1`**. Spec validation errors → **exit 1**. | [eikopf/tracey](https://github.com/eikopf/tracey), **`.config/tracey/config.styx`**, workspace daemon |
| **`scripts/run_mutants.sh`** | `cargo mutants` using **`.cargo/mutants.toml`** | `cargo-mutants` |
| **`scripts/fuzz_csv.sh`** | `cargo fuzz run csv_parse` in **`fuzz/`** | nightly toolchain, `cargo-fuzz` |
| **`scripts/fuzz_pipeline.sh`** | `cargo fuzz run my_target` in **`fuzz/`** | same |
| **`scripts/fuzz_coverage.sh`** | `cargo fuzz coverage my_target` + llvm-cov summary | `llvm-tools-preview` |
| **`scripts/run_tracy_profile.sh`** | Prints Tracy / **`graph`** profiling hints (no hard dependency) | optional Tracy + `tracet` |
| **`scripts/ci_local_default.sh`** | Sequential: fmt → clippy default → nextest default → coverage → provider check → tracey script | all of the above as needed |
| **`scripts/ci_local_schwab.sh`** | Clippy schwab + nextest schwab | same |

Further prerequisites and install hints: [`scripts/README.md`](scripts/README.md).

r[test.tracey.coverage]

- 100 % of `r[...]` tags in this spec must be implemented **and** verified in tests.  
- **tracey** reports / dashboard are the intended talk wrap-up once coverage is complete.

r[test.tracey.workflow]

Repository tracey process and CI gate.
- For **every** `r[tag]` appearing in this specification document, production code must carry **`// r[impl <tag>]`** (with a **space** after `r[` — tracey’s verb syntax) at the site that satisfies the requirement (or the narrowest enclosing module), and tests must carry **`// r[verify <tag>]`** on the test function (or module) that proves the behavior. See the **“Reference requirements in your code”** section in the [tracey README](https://github.com/eikopf/tracey).
- **Gate:** CI **fails** if any spec tag lacks a matching **impl** and **verify** pair (**mirrors** **`r[test.tracey.coverage]`**).
- **Artifact / command:** run **`./scripts/tracey_report.sh`**, which invokes **`tracey query validate`** and **`tracey query status`** against **`.config/tracey/config.styx`**. Use **`tracey web`** locally for the HTML dashboard if desired. If **`tracey`** is not on **`PATH`**, the script **exits 0** (optional in CI). For daemon or **`query`** transport errors, set **`STOCKVIZ_TRACEY_STRICT=1`** to fail the step; spec/schema validation errors always fail the script when **`tracey`** runs successfully.

r[test.tracing]

Major functions (download, load CSV, SMA calc, **price + volume render**, zoom, resize) expose **`tracing::span!` / `event!`** for a live Tracy profiler demo. This complements—does not replace—**`log` + `flexi_logger`** (see §5). For a local Tracy-oriented checklist, run **`./scripts/run_tracy_profile.sh`**.

---

### 7. Non-Functional (Keep It Simple)

r[app.simple]

Zero persistent state except CSV files; minimal dependencies; native Ubuntu.
- Zero persistent state except the `.csv` files  
- Handles 20+ years of data (~7000 bars) smoothly at 60 fps  
- Runs natively on Ubuntu (no web/Android needed for talk)  
- User-visible failures follow **§4 Errors & Process Exit** (`Result` propagation, top-level `error!`, non-zero exit)  
- **Minimal dependencies:** avoid unnecessary crates; **`flexi_logger` is required** for logging init.  
- started with: cargo init stockviz

---

### 8. Dependencies (informative)

r[app.deps]

Informative dependency intent (clap, chrono-tz, csv, reqwest, thiserror, etc.).
Expected crates beyond **`r[app.identity]`** pinned names (versions: **latest compatible at talk date** unless pinned in `Cargo.lock`):

| Crate / area        | Role |
|---------------------|------|
| `clap` (derive)     | **`r[cli.commands]`** |
| `chrono` or `time` + TZ DB (`chrono-tz` / `iana-time-zone` / equivalent) | **`America/New_York`**, `D_cal`, CSV dates |
| `csv`               | Read/write **`r[data.format]`** |
| HTTP client (e.g. `reqwest` blocking or async—**implementation choice**) | Twelve Data / future Schwab |
| `serde` (optional where useful) | JSON/CSV helpers if needed |
| `thiserror`         | **Preferred** project `Error` type (**§4**) |

Pin or swap alternatives in `Cargo.toml` as needed; this table is **normative intent**, not a frozen lockfile.
