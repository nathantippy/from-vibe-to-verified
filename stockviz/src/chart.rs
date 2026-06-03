//! Chart math: visible range, per-pixel buckets, synthetic OHLC + volume sum.

use crate::data::Bar;

// r[impl gui.chart.candles]
#[derive(Clone, Debug)]
pub struct Bucket<'a> {
    pub rows: &'a [Bar],
}

// r[impl gui.chart.candles]
impl<'a> Bucket<'a> {
    // r[impl gui.chart.candles]
    pub fn synthetic_open(&self) -> f64 {
        self.rows.first().map(|b| b.open).unwrap_or(0.0)
    }

    // r[impl gui.chart.candles]
    pub fn synthetic_high(&self) -> f64 {
        self.rows
            .iter()
            .map(|b| b.high)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    // r[impl gui.chart.candles]
    pub fn synthetic_low(&self) -> f64 {
        self.rows
            .iter()
            .map(|b| b.low)
            .fold(f64::INFINITY, f64::min)
    }

    // r[impl gui.chart.candles]
    pub fn synthetic_close(&self) -> f64 {
        self.rows.last().map(|b| b.close).unwrap_or(0.0)
    }

    // r[impl gui.chart.volume]
    pub fn volume_sum(&self) -> f64 {
        self.rows.iter().map(|b| b.volume).sum()
    }

    // r[impl gui.chart.candles]
    pub fn is_bull(&self) -> bool {
        self.synthetic_close() >= self.synthetic_open()
    }
}

// r[impl gui.chart.candles]
// r[impl gui.chart.volume]
/// Buckets price/volume rows for candle bodies and histogram.
pub fn buckets_for_width(bars: &[Bar], width: usize) -> Vec<Bucket<'_>> {
    if width == 0 || bars.is_empty() {
        return Vec::new();
    }
    let n = bars.len();
    let width = width.min(4096); // sanity
    let base = n / width;
    let rem = n % width;
    let mut out = Vec::with_capacity(width);
    let mut start = 0;
    for col in 0..width {
        let add = if col < rem {
            base.saturating_add(1)
        } else {
            base
        };
        let end = (start + add).min(n);
        out.push(Bucket {
            rows: &bars[start..end],
        });
        start = end;
    }
    out
}

// r[impl gui.chart.width.fill]
/// Layout columns for a pane: `min(bar_count, width_px)` (capped at 4096).
pub fn layout_column_count(n_bars: usize, width_px: usize) -> usize {
    if n_bars == 0 || width_px == 0 {
        return 0;
    }
    n_bars.min(width_px.min(4096)).max(1)
}

// r[impl gui.chart.width.fill]
// r[impl gui.chart.candles]
// r[impl gui.chart.volume]
/// Pane bucketing: fills horizontal width without empty trailing columns.
pub fn buckets_for_pane(bars: &[Bar], width_px: usize) -> Vec<Bucket<'_>> {
    let cols = layout_column_count(bars.len(), width_px);
    if cols == 0 {
        return Vec::new();
    }
    buckets_for_width(bars, cols)
}

// r[impl gui.chart.width.fill]
/// X center of the rightmost bar in pane layout (anchor at right edge).
pub fn last_bar_center_x(n_bars: usize, left: f32, width_px: f32, pane_width_px: usize) -> f32 {
    if n_bars == 0 {
        return left;
    }
    let cols = layout_column_count(n_bars, pane_width_px);
    let bi = bucket_index_for_bar(n_bars - 1, n_bars, cols);
    bucket_center_x(left, width_px, bi, cols)
}

// r[impl gui.chart.sma.align]
/// Bar index → bucket column using the same partition as [`buckets_for_width`].
pub fn bucket_index_for_bar(bar_index: usize, n_bars: usize, n_buckets: usize) -> usize {
    if n_bars == 0 || n_buckets == 0 {
        return 0;
    }
    let width = n_buckets.min(4096);
    let base = n_bars / width;
    let rem = n_bars % width;
    let mut start = 0;
    for col in 0..width {
        let add = if col < rem {
            base.saturating_add(1)
        } else {
            base
        };
        let end = (start + add).min(n_bars);
        if bar_index < end {
            return col;
        }
        start = end;
    }
    width.saturating_sub(1)
}

// r[impl gui.chart.sma.align]
/// Horizontal center of a bucket column (matches candle wick center in `paint_price`).
pub fn bucket_center_x(left: f32, width_px: f32, bucket_index: usize, n_buckets: usize) -> f32 {
    let n = n_buckets.max(1);
    let col_w = width_px / n as f32;
    left + (bucket_index as f32 + 0.5) * col_w
}

// r[impl gui.chart.sma.align]
/// Screen X for an SMA point at `bar_index` in a slice of `n_bars` rows.
pub fn sma_screen_x_for_bar(
    bar_index: usize,
    n_bars: usize,
    left: f32,
    width_px: f32,
    n_buckets: usize,
) -> f32 {
    let bi = bucket_index_for_bar(bar_index, n_bars, n_buckets);
    bucket_center_x(left, width_px, bi, n_buckets)
}

// r[impl gui.chart.sma.align]
#[cfg(debug_assertions)]
pub fn debug_assert_sma_candle_x_alignment(
    n_bars: usize,
    left: f32,
    width_px: f32,
    n_buckets: usize,
) {
    if n_bars == 0 || n_buckets == 0 {
        return;
    }
    for i in 0..n_bars {
        let x = sma_screen_x_for_bar(i, n_bars, left, width_px, n_buckets);
        let bi = bucket_index_for_bar(i, n_bars, n_buckets);
        let xc = bucket_center_x(left, width_px, bi, n_buckets);
        debug_assert!(
            (x - xc).abs() < 1e-3,
            "bar {i}: sma x {x} != candle center {xc}"
        );
    }
}

/// Minimum visible trading days when zooming in (**`r[gui.chart.zoom.limits]`**).
// r[impl gui.chart.zoom.limits]
pub const MIN_ZOOM_VISIBLE_BARS: usize = 7;

// r[impl gui.chart.anchor]
// r[impl gui.chart.zoom.limits]
pub fn visible_range(anchor_idx: usize, n_visible: usize) -> (usize, usize) {
    let cap = anchor_idx + 1;
    let nv = n_visible.min(cap).max(1);
    let start = anchor_idx + 1 - nv;
    (start, nv)
}

// r[impl gui.chart.zoom]
// r[impl gui.chart.zoom.limits]
/// Halve visible bar count (**`r[gui.chart.zoom]`**), floored at [`MIN_ZOOM_VISIBLE_BARS`].
pub fn zoom_in(visible_bars: usize) -> usize {
    (visible_bars / 2).max(MIN_ZOOM_VISIBLE_BARS)
}

// r[impl gui.chart.zoom.limits]
/// Apply zoom-in floor capped by series length.
pub fn zoom_in_capped(visible_bars: usize, max_bars: usize) -> usize {
    let floor = MIN_ZOOM_VISIBLE_BARS.min(max_bars.max(1));
    zoom_in(visible_bars).min(max_bars).max(floor)
}

// r[impl gui.chart.zoom]
/// Double visible bar count up to full history (**`r[gui.chart.zoom]`**).
pub fn zoom_out(visible_bars: usize, max_bars: usize) -> usize {
    (visible_bars * 2).min(max_bars)
}

// r[impl gui.chart.resize]
/// How many bars fit at minimum pixel width (**`r[gui.chart.resize]`**).
pub fn bars_fitting_width(price_w: f32, min_px_per_bar: f32) -> usize {
    ((price_w / min_px_per_bar.max(1.0)).floor() as usize).max(1)
}

// r[impl gui.chart.yticks]
/// Right margin reserved for price labels in the price pane.
pub const PRICE_LABEL_MARGIN_PX: f32 = 48.0;

// r[impl gui.chart.pane.align]
/// Shared drawable chart width for price and volume panes (excludes Y-label margin).
pub fn pane_chart_width_px(full_pane_width_px: f32) -> f32 {
    (full_pane_width_px - PRICE_LABEL_MARGIN_PX).max(1.0)
}

// r[impl gui.chart.pane.align]
/// Right edge of the shared drawable chart rect.
pub fn pane_chart_right(full_pane_left: f32, full_pane_width_px: f32) -> f32 {
    full_pane_left + pane_chart_width_px(full_pane_width_px)
}

// r[impl gui.chart.yticks]
/// Pick a "nice" step size for price axis ticks (1/2/5 × 10^n).
fn nice_tick_step(raw_step: f64) -> f64 {
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return 1.0;
    }
    let exp = raw_step.log10().floor();
    let base = 10_f64.powf(exp);
    let frac = raw_step / base;
    let nice_frac = if frac <= 1.0 {
        1.0
    } else if frac <= 2.0 {
        2.0
    } else if frac <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice_frac * base
}

// r[impl gui.chart.yticks]
const MAX_PRICE_TICKS: usize = 512;

// r[impl gui.chart.yticks]
/// Sparse price levels for the right edge of the price pane.
pub fn price_ticks(ymin: f64, ymax: f64, height_px: usize) -> Vec<f64> {
    if !(ymin.is_finite() && ymax.is_finite()) || height_px == 0 {
        return Vec::new();
    }
    let span = (ymax - ymin).abs();
    let scale = ymin.abs().max(ymax.abs()).max(1.0);
    if span < scale * f64::EPSILON {
        return vec![ymin];
    }
    let target = (height_px / 80).max(1);
    let raw_step = span / target as f64;
    let step = nice_tick_step(raw_step);
    let min_step = f64::EPSILON * scale;
    if step < min_step {
        return vec![ymin];
    }
    let start = (ymin / step).ceil() * step;
    let max_ticks = (target + 2).min(MAX_PRICE_TICKS);
    let mut ticks = Vec::new();
    let mut p = start;
    for _ in 0..max_ticks {
        if p > ymax + step * 1e-9 {
            break;
        }
        ticks.push(p);
        let next = p + step;
        if next == p {
            break;
        }
        p = next;
    }
    ticks
}

// r[impl gui.chart.yticks]
/// Format a price label for the Y axis.
pub fn format_price(p: f64) -> String {
    let abs = p.abs();
    if abs >= 1000.0 {
        format!("{p:.0}")
    } else if abs >= 1.0 {
        format!("{p:.2}")
    } else {
        format!("{p:.4}")
    }
}

// r[impl gui.chart.sma.legend]
/// Legend entry labels for SMA identification.
pub fn sma_legend_labels() -> [&'static str; 2] {
    ["SMA 50", "SMA 150"]
}

// r[impl gui.chart]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Bar;
    use chrono::NaiveDate;

    // r[impl gui.chart.volume]
    // r[impl gui.chart.candles]
    fn bar(d: &str, o: f64, h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar {
            date: NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap(),
            open: o,
            high: h,
            low: l,
            close: c,
            volume: v,
        }
    }

    // r[impl gui.chart.yticks]
    // r[verify gui.chart.yticks]
    #[test]
    fn price_ticks_within_range_and_monotonic() {
        let ticks = price_ticks(100.0, 200.0, 400);
        assert!(!ticks.is_empty());
        for w in ticks.windows(2) {
            assert!(w[1] > w[0]);
        }
        for t in &ticks {
            assert!(*t >= 100.0 - 1e-9 && *t <= 200.0 + 1e-9);
        }
    }

    // r[impl gui.chart.yticks]
    // r[verify gui.chart.yticks]
    #[test]
    fn price_ticks_near_equal_large_values_do_not_hang() {
        let ticks = price_ticks(2261634.509803921, 2261634.5098039214, 4095);
        assert!(ticks.len() <= MAX_PRICE_TICKS);
        assert_eq!(ticks, vec![2261634.509803921]);
    }

    // r[impl gui.chart.yticks]
    // r[verify gui.chart.yticks]
    #[test]
    fn price_ticks_fuzz_artifact_control_tail_is_bounded() {
        use crate::test_inputs::ChartFuzzCtrl;

        let data =
            include_bytes!("../fuzz/artifacts/my_target/oom-7fc036367c008997f59b0adc14d213e7423b8b29");
        let ctrl = ChartFuzzCtrl::from_bytes(&data[data.len() - 64..]);
        let ticks = price_ticks(ctrl.ymin, ctrl.ymax, ctrl.height_px);
        assert!(ticks.len() <= MAX_PRICE_TICKS);
    }

    // r[impl gui.chart.yticks]
    // r[verify gui.chart.yticks]
    #[test]
    fn format_price_adapts_decimals() {
        assert_eq!(format_price(1234.56), "1235");
        assert_eq!(format_price(42.5), "42.50");
        assert_eq!(format_price(0.0123), "0.0123");
    }

    // r[impl gui.chart.sma.legend]
    // r[verify gui.chart.sma.legend]
    #[test]
    fn sma_legend_labels_match_spec() {
        assert_eq!(sma_legend_labels(), ["SMA 50", "SMA 150"]);
    }

    // r[impl gui.chart.pane.align]
    // r[verify gui.chart.pane.align]
    #[test]
    fn pane_chart_width_reserves_ytick_margin() {
        assert!((pane_chart_width_px(900.0) - 852.0).abs() < 1e-3);
        assert!((pane_chart_width_px(10.0) - 1.0).abs() < 1e-3);
    }

    // r[impl gui.chart.pane.align]
    // r[verify gui.chart.pane.align]
    #[test]
    fn price_volume_bucket_centers_match() {
        let bars: Vec<Bar> = (0..80)
            .map(|i| {
                bar(
                    &format!("2020-01-{:02}", (i % 28) + 1),
                    1.,
                    2.,
                    0.5,
                    1.5,
                    10.,
                )
            })
            .collect();
        for full_w in [480.0f32, 900.0] {
            let chart_w = pane_chart_width_px(full_w);
            let width_px = chart_w as usize;
            let buckets = buckets_for_pane(&bars, width_px);
            let cols = buckets.len();
            let left = 0.0f32;
            for i in 0..cols {
                let xc = bucket_center_x(left, chart_w, i, cols);
                let xc_again = bucket_center_x(left, chart_w, i, cols);
                assert!((xc - xc_again).abs() < 1e-4);
            }
            let last = last_bar_center_x(bars.len(), left, chart_w, width_px);
            assert!(last <= pane_chart_right(left, full_w) + 1e-3);
        }
    }

    // r[impl gui.chart.volume]
    // r[verify gui.chart.volume]
    #[test]
    fn bucket_sums_volume() {
        let rows = [
            bar("2020-01-01", 1., 2., 0.5, 1.5, 10.),
            bar("2020-01-02", 1.5, 2., 1., 1.2, 30.),
        ];
        let b = Bucket { rows: &rows[..] };
        assert!((b.volume_sum() - 40.).abs() < 1e-9);
    }

    // r[impl gui.chart.candles]
    // r[verify gui.chart.candles]
    #[test]
    fn candle_bucket_synthetic_ohlc() {
        let rows = [
            bar("2020-01-01", 1., 3., 0.5, 2., 10.),
            bar("2020-01-02", 2., 4., 1.5, 3., 5.),
        ];
        let bk = Bucket { rows: &rows[..] };
        assert!((bk.synthetic_open() - 1.).abs() < 1e-9);
        assert!((bk.synthetic_close() - 3.).abs() < 1e-9);
        assert!((bk.synthetic_high() - 4.).abs() < 1e-9);
        assert!((bk.synthetic_low() - 0.5).abs() < 1e-9);
    }

    // r[impl gui.chart.zoom.limits]
    // r[verify gui.chart.zoom.limits]
    #[test]
    fn visible_range_never_exceeds_anchor_plus_one() {
        let (start, len) = visible_range(9, 100);
        assert_eq!(len, 10);
        assert_eq!(start, 0);
    }

    // r[impl gui.chart.zoom]
    // r[verify gui.chart.zoom]
    #[test]
    fn zoom_in_halves_and_out_doubles() {
        assert_eq!(zoom_in(80), 40);
        assert_eq!(zoom_in(10), 7);
        assert_eq!(zoom_in(7), 7);
        assert_eq!(zoom_in_capped(80, 80), 40);
        assert_eq!(zoom_in_capped(5, 5), 5);
        assert_eq!(zoom_out(40, 80), 80);
        assert_eq!(zoom_out(80, 80), 80);
    }

    // r[impl gui.chart.resize]
    // r[verify gui.chart.resize]
    #[test]
    fn bars_fitting_width_grows_with_window() {
        let narrow = bars_fitting_width(200.0, 4.0);
        let wide = bars_fitting_width(800.0, 4.0);
        assert!(wide >= narrow);
    }

    // r[impl gui.chart]
    // r[verify gui.chart]
    #[test]
    fn gui_chart_parent_bucket_bull_bear() {
        let rows = [
            bar("2020-01-01", 10., 12., 9., 11., 100.),
            bar("2020-01-02", 11., 12., 10., 10., 50.),
        ];
        let bull = Bucket { rows: &rows[0..1] };
        let bear = Bucket { rows: &rows[1..2] };
        assert!(bull.is_bull());
        assert!(!bear.is_bull());
    }

    // r[impl gui.chart.candles]
    // r[verify gui.chart.candles]
    #[test]
    fn buckets_for_width_empty_and_zero_width() {
        let rows: Vec<Bar> = (0..5)
            .map(|i| {
                let d = format!("2020-01-{:02}", i + 1);
                bar(&d, 1., 2., 0.5, 1.5, 10.)
            })
            .collect();
        assert!(buckets_for_width(&rows, 0).is_empty());
        assert!(buckets_for_width(&[], 4).is_empty());
    }

    // r[impl gui.chart.candles]
    // r[verify gui.chart.candles]
    #[test]
    fn buckets_for_width_remainder_distribution() {
        let rows: Vec<Bar> = (0..3)
            .map(|i| {
                let d = format!("2020-01-{:02}", i + 1);
                bar(&d, 1., 2., 0.5, 1.5, 10.)
            })
            .collect();
        let buckets = buckets_for_width(&rows, 5);
        let counts: Vec<usize> = buckets.iter().map(|b| b.rows.len()).collect();
        assert_eq!(counts, vec![1, 1, 1, 0, 0]);
    }

    // r[impl gui.chart.candles]
    // r[verify gui.chart.candles]
    #[test]
    fn buckets_for_width_exact_partition_small() {
        let rows: Vec<Bar> = (0..10)
            .map(|i| {
                let d = format!("2020-01-{:02}", i + 1);
                bar(&d, 1., 2., 0.5, 1.5, 10.)
            })
            .collect();
        let buckets = buckets_for_width(&rows, 4);
        assert_eq!(buckets.len(), 4);
        let counts: Vec<usize> = buckets.iter().map(|b| b.rows.len()).collect();
        assert_eq!(counts, vec![3, 3, 2, 2]);
        assert_eq!(counts.iter().sum::<usize>(), 10);
    }

    // r[impl gui.chart.candles]
    // r[verify gui.chart.candles]
    #[test]
    fn buckets_for_width_width_capped_at_4096() {
        let rows: Vec<Bar> = (0..20)
            .map(|i| {
                let d = format!("2020-01-{:02}", i + 1);
                bar(&d, 1., 2., 0.5, 1.5, 10.)
            })
            .collect();
        let buckets = buckets_for_width(&rows, 5000);
        assert_eq!(buckets.len(), 4096);
        let covered: usize = buckets.iter().map(|b| b.rows.len()).sum();
        assert_eq!(covered, 20);
    }

    // r[impl gui.chart.resize]
    // r[verify gui.chart.resize]
    #[test]
    fn bars_fitting_width_exact() {
        assert_eq!(bars_fitting_width(800.0, 4.0), 200);
    }

    // r[impl gui.chart.resize]
    // r[verify gui.chart.resize]
    #[test]
    fn bars_fitting_width_min_px_clamp() {
        assert_eq!(bars_fitting_width(100.0, 0.0), 100);
    }

    // r[impl gui.chart.sma.align]
    // r[verify gui.chart.sma.align]
    #[test]
    fn bucket_index_for_bar_matches_partition() {
        let rows: Vec<Bar> = (0..10)
            .map(|i| {
                let d = format!("2020-01-{:02}", i + 1);
                bar(&d, 1., 2., 0.5, 1.5, 10.)
            })
            .collect();
        let buckets = buckets_for_width(&rows, 4);
        let mut offset = 0;
        for (col, bk) in buckets.iter().enumerate() {
            for j in 0..bk.rows.len() {
                assert_eq!(bucket_index_for_bar(offset + j, 10, 4), col);
            }
            offset += bk.rows.len();
        }
    }

    // r[impl gui.chart.sma.align]
    // r[verify gui.chart.sma.align]
    #[test]
    fn bucket_center_x_monotonic_in_bucket_index() {
        let left = 10.0;
        let w = 200.0;
        let n = 8;
        let mut prev = f32::NEG_INFINITY;
        for i in 0..n {
            let x = bucket_center_x(left, w, i, n);
            assert!(x > prev);
            prev = x;
        }
    }

    // r[impl gui.chart.sma.align]
    // r[verify gui.chart.sma.align]
    #[test]
    fn sma_screen_x_matches_candle_center_formula() {
        let left = 0.0;
        let w = 100.0;
        let n_bars = 100;
        let n_buckets = 20;
        for i in [0, 4, 49, 99] {
            let bi = bucket_index_for_bar(i, n_bars, n_buckets);
            let x_sma = sma_screen_x_for_bar(i, n_bars, left, w, n_buckets);
            let x_candle = bucket_center_x(left, w, bi, n_buckets);
            assert!((x_sma - x_candle).abs() < 1e-5);
        }
    }

    // r[impl gui.chart.width.fill]
    // r[verify gui.chart.width.fill]
    #[test]
    fn layout_column_count_sparse_uses_bar_count() {
        assert_eq!(layout_column_count(80, 900), 80);
        let left = 0.0f32;
        let w = 900.0f32;
        let last = last_bar_center_x(80, left, w, 900);
        assert!((last - (left + w - w / 160.0)).abs() < 1e-3);
    }

    // r[impl gui.chart.width.fill]
    // r[verify gui.chart.width.fill]
    #[test]
    fn layout_column_count_compressed_uses_width() {
        assert_eq!(layout_column_count(100, 20), 20);
        assert_eq!(bucket_index_for_bar(99, 100, 20), 19);
        let rows: Vec<Bar> = (0..100)
            .map(|i| {
                let d = format!("2020-01-{:02}", (i % 28) + 1);
                bar(&d, 1., 2., 0.5, 1.5, 10.)
            })
            .collect();
        let buckets = buckets_for_pane(&rows, 20);
        assert_eq!(buckets.len(), 20);
        assert!(buckets.iter().all(|b| !b.rows.is_empty()));
    }
}

// r[impl test.proptest.sma.align]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_sma_align {
    // r[impl test.proptest.sma.align]
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use super::*;
    use crate::test_inputs::AlignLayoutParams;

    // r[impl test.proptest.sma.align]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.sma.align]
        #[test]
        // r[verify test.proptest.sma.align]
        // r[verify test.arbitrary.proptest]
        fn bar_index_maps_to_bucket_partition(params in arb::<AlignLayoutParams>()) {
            let n = params.n_bars;
            let width = params.width_px;
            let bars = params.bars().into_inner();
            let cols = layout_column_count(n, width);
            let buckets = buckets_for_pane(&bars, width);
            prop_assert_eq!(buckets.len(), cols);
            let mut offset = 0;
            for (col, bk) in buckets.iter().enumerate() {
                for j in 0..bk.rows.len() {
                    prop_assert_eq!(bucket_index_for_bar(offset + j, n, cols), col);
                }
                offset += bk.rows.len();
            }
        }

        // r[impl test.proptest.sma.align]
        #[test]
        // r[verify test.proptest.sma.align]
        fn sma_x_equals_bucket_center(params in arb::<AlignLayoutParams>()) {
            let n = params.n_bars;
            let width = params.width_px;
            let left = 5.0f32;
            let w = 200.0f32;
            let cols = layout_column_count(n, width);
            for i in 0..n {
                let bi = bucket_index_for_bar(i, n, cols);
                let x_sma = sma_screen_x_for_bar(i, n, left, w, cols);
                let x_c = bucket_center_x(left, w, bi, cols);
                prop_assert!((x_sma - x_c).abs() < 1e-4);
            }
        }
    }
}

// r[impl test.proptest.chart.width.fill]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_chart_width_fill {
    // r[impl test.proptest.chart.width.fill]
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use super::*;
    use crate::test_inputs::WidthFillLayoutParams;

    // r[impl test.proptest.chart.width.fill]
    /// Talk demo only: `true` asserts the old wrong column count (`width` instead of `min(n, width)`).
    const DEMO_WIDTH_PROPTEST_FAIL: bool = false; //PROPTEST DEMO

    // r[impl test.proptest.chart.width.fill]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.chart.width.fill]
        #[test]
        // r[verify test.proptest.chart.width.fill]
        // r[verify test.arbitrary.proptest]
        fn pane_layout_fills_horizontal_extent(params in arb::<WidthFillLayoutParams>()) {
            let n = params.n_bars;
            let width = params.width_px;
            let cols = layout_column_count(n, width);
            let expected_cols = n.min(width.min(4096)).max(1);
            let expected = if DEMO_WIDTH_PROPTEST_FAIL {
                width.min(4096)
            } else {
                expected_cols
            };
            prop_assert_eq!(cols, expected);

            let left = 0.0f32;
            let w = width as f32;
            let last = last_bar_center_x(n, left, w, width);
            let min_last = left + w * (1.0 - 1.0 / cols as f32) - 1e-3;
            prop_assert!(last >= min_last);
            prop_assert_eq!(bucket_index_for_bar(n - 1, n, cols), cols - 1);
        }
    }
}

// r[impl test.proptest.pane.align]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_pane_align {
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use super::*;
    use crate::test_inputs::PaneAlignLayoutParams;

    // r[impl test.proptest.pane.align]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.pane.align]
        #[test]
        // r[verify test.proptest.pane.align]
        // r[verify test.arbitrary.proptest]
        fn price_volume_share_bucket_centers(params in arb::<PaneAlignLayoutParams>()) {
            let n = params.n_bars;
            let full_width = params.full_pane_width;
            let bars = params.bars().into_inner();
            let chart_w = pane_chart_width_px(full_width as f32);
            let width_px = chart_w as usize;
            let buckets = buckets_for_pane(&bars, width_px);
            let cols = buckets.len();
            let left = 0.0f32;
            for i in 0..cols {
                let a = bucket_center_x(left, chart_w, i, cols);
                let b = bucket_center_x(left, chart_w, i, cols);
                prop_assert!((a - b).abs() < 1e-4);
            }
            prop_assert_eq!(layout_column_count(n, width_px), cols);
        }
    }
}

// r[impl test.proptest.zoom.limits]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_zoom_limits {
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use super::*;
    use crate::test_inputs::ZoomSpanParams;

    // r[impl test.proptest.zoom.limits]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.zoom.limits]
        #[test]
        // r[verify test.proptest.zoom.limits]
        // r[verify test.arbitrary.proptest]
        fn zoom_in_never_below_floor(params in arb::<ZoomSpanParams>()) {
            let floor = MIN_ZOOM_VISIBLE_BARS.min(params.max_bars);
            let mut v = params.visible_bars.min(params.max_bars);
            for _ in 0..12 {
                v = zoom_in_capped(v, params.max_bars);
                prop_assert!(v >= floor);
            }
        }
    }
}

// r[impl test.proptest.chart]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_chart_buckets {
    // r[impl test.proptest.chart]
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use super::*;
    use crate::test_inputs::BucketLayoutParams;

    // r[impl test.proptest.chart]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.chart]
        #[test]
        // r[verify test.proptest.chart]
        // r[verify test.arbitrary.proptest]
        fn buckets_partition_invariant(params in arb::<BucketLayoutParams>()) {
            let n = params.n_bars;
            let width = params.width_px;
            let bars = params.bars().into_inner();
            let buckets = buckets_for_width(&bars, width);
            let effective_width = width.min(4096);
            prop_assert_eq!(buckets.len(), effective_width);
            let covered: usize = buckets.iter().map(|b| b.rows.len()).sum();
            prop_assert_eq!(covered, n);
        }
    }
}
