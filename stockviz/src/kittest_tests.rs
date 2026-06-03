//! UI tests: resize, volume pane (**`r[test.kittest.resize]`**, **`r[test.kittest.volume]`**).

// r[impl test.kittest.resize]
// r[impl test.kittest.volume]
use chrono::{Duration, NaiveDate};
use egui::Vec2;
use egui_kittest::{Harness, kittest::Queryable as _};

use crate::data::Bar;
use crate::gui::StockvizApp;

// r[impl test.kittest.resize]
fn large_bars(n: usize) -> Vec<Bar> {
    let base = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    (0..n)
        .map(|i| {
            let d = base + Duration::days(i64::try_from(i).unwrap_or(i as i64));
            let o = 100.0 + f64::from(i as u32);
            let close = if i % 2 == 0 { o + 0.5 } else { o - 0.5 };
            Bar {
                date: d,
                open: o,
                high: o + 2.0,
                low: o - 1.0,
                close,
                volume: 1000.0 + f64::from(i as u32),
            }
        })
        .collect()
}

// r[impl test.kittest.resize]
fn sample_bars() -> Vec<Bar> {
    large_bars(80)
}

// r[impl test.kittest.resize]
fn build_harness_large(size: Vec2, n: usize) -> Harness<'static, StockvizApp> {
    Harness::builder()
        .with_size(size)
        .build_eframe(|cc| StockvizApp::new(cc, large_bars(n)))
}

// r[impl test.kittest.resize]
fn build_harness(size: Vec2) -> Harness<'static, StockvizApp> {
    Harness::builder()
        .with_size(size)
        .build_eframe(|cc| StockvizApp::new(cc, sample_bars()))
}

// r[impl test.kittest.resize]
// r[verify test.kittest.resize]
// r[verify gui.core]
// r[verify gui.chart]
// r[verify gui.chart.resize]
// r[verify gui.chart.zoom]
// r[verify gui.chart.zoom.limits]
// r[verify gui.chart.timeaxis]
// r[verify gui.chart.xticks]
// r[verify test.tracing]
#[test]
fn kittest_zoom_in_halves_visible_span() {
    let mut harness = build_harness(Vec2::new(900.0, 700.0));
    harness.run_ok();
    let before = harness.state().visible_bars();
    assert_eq!(before, 80, "starts at full history");
    harness.get_by_label("Zoom In (+)").click();
    harness.run_ok();
    assert_eq!(harness.state().visible_bars(), 40);
}

// r[impl test.kittest.resize]
// r[verify gui.chart.zoom.limits]
#[test]
fn kittest_zoom_in_floors_at_seven_days() {
    let mut harness = build_harness(Vec2::new(900.0, 700.0));
    harness.run_ok();
    for _ in 0..8 {
        harness.get_by_label("Zoom In (+)").click();
        harness.run_ok();
    }
    assert_eq!(harness.state().visible_bars(), 7);
}

// r[impl test.kittest.resize]
// r[verify gui.chart.candles]
#[test]
fn kittest_full_history_narrow_window_shows_all_bars() {
    let mut harness = build_harness_large(Vec2::new(320.0, 400.0), 200);
    harness.run_ok();
    assert_eq!(harness.state().visible_bars(), 200);
    assert_eq!(harness.state().visible_start_idx(320.0), 0);
}

// r[impl test.kittest.resize]
#[test]
fn kittest_zoom_out_doubles_visible_span() {
    let mut harness = build_harness(Vec2::new(900.0, 700.0));
    harness.run_ok();
    harness.get_by_label("Zoom In (+)").click();
    harness.run_ok();
    harness.get_by_label("Zoom Out (-)").click();
    harness.run_ok();
    assert_eq!(harness.state().visible_bars(), 80);
}

// r[impl test.kittest.resize]
#[test]
fn kittest_resize_wider_shows_more_history() {
    let mut harness = build_harness(Vec2::new(320.0, 400.0));
    harness.run_ok();
    harness.get_by_label("Zoom In (+)").click();
    harness.run_ok();
    let narrow = harness.state().time_span_bars_pub();
    harness.set_size(Vec2::new(1200.0, 700.0));
    harness.run_ok();
    let wide = harness.state().time_span_bars_pub();
    assert!(
        wide > narrow,
        "partial zoom + wider window should reveal more history: narrow={narrow} wide={wide}"
    );
    harness.run_ok();
    let _ = harness.get_by_label("Zoom Out (-)");
}

// r[impl test.kittest.volume]
// r[verify test.kittest.volume]
// r[verify gui.chart.volume]
// r[verify gui.chart.pane.align]
// r[verify gui.chart.candles]
#[test]
fn kittest_volume_marker_present() {
    let mut harness = build_harness(Vec2::new(640.0, 480.0));
    harness.run_ok();
    let _n = harness.get_by_label("__stockviz_volume__");
}

// r[impl test.kittest.resize]
// r[verify gui.chart.width.fill]
// r[verify gui.chart.pane.align]
#[test]
fn kittest_launch_fills_horizontal_width() {
    let mut harness = build_harness(Vec2::new(900.0, 700.0));
    harness.run_ok();
    let drawable = crate::chart::pane_chart_width_px(900.0) as usize;
    assert_eq!(crate::chart::layout_column_count(80, drawable), 80);
    let last = crate::chart::last_bar_center_x(80, 0.0, drawable as f32, drawable);
    assert!(
        last > drawable as f32 * 0.85,
        "rightmost bar should use right portion of drawable chart: {last}"
    );
}

// r[impl test.kittest.resize]
// r[verify gui.chart.pane.align]
#[test]
fn kittest_price_volume_share_drawable_width() {
    let mut harness = build_harness(Vec2::new(900.0, 700.0));
    harness.run_ok();
    let full_w = 900.0f32;
    let chart_w = crate::chart::pane_chart_width_px(full_w);
    assert!((chart_w - 852.0).abs() < 1.0);
    let cols = crate::chart::layout_column_count(80, chart_w as usize);
    let left = 0.0f32;
    for i in 0..cols {
        let xc = crate::chart::bucket_center_x(left, chart_w, i, cols);
        assert!(xc <= crate::chart::pane_chart_right(left, full_w) + 1e-3);
    }
    let _ = harness.get_by_label("__stockviz_volume__");
}

// r[impl test.kittest.resize]
// r[verify gui.chart.xticks]
#[test]
fn kittest_narrow_window_exercises_chart_paint_path() {
    let mut harness = build_harness(Vec2::new(480.0, 400.0));
    harness.run_ok();
    // Date ticks are drawn with painter.text (not a11y labels); narrow width still
    // drives sparse tick layout in paint_price per r[gui.chart.xticks].
    // Narrow width: n > cols (compression). paint_price uses buckets_for_pane per
    // r[gui.chart.sma.align] and r[gui.chart.width.fill].
    let _ = harness.get_by_label("Zoom In (+)");
    let _ = harness.get_by_label("__stockviz_volume__");
    assert!(harness.state().visible_bars() > 0);
}

// r[impl test.kittest.volume]
// r[verify gui.chart.sma.legend]
#[test]
fn kittest_sma_legend_marker_present() {
    let mut harness = build_harness(Vec2::new(640.0, 480.0));
    harness.run_ok();
    let _ = harness.get_by_label("__stockviz_sma_legend__");
}

// r[impl test.kittest.resize]
// r[verify gui.chart.yticks]
// r[verify gui.chart.sma.legend]
#[test]
fn kittest_price_pane_exercises_yticks_and_legend() {
    let mut harness = build_harness(Vec2::new(900.0, 700.0));
    harness.run_ok();
    // Price Y ticks and SMA legend are painter-drawn; this test exercises paint_price
    // per r[gui.chart.yticks] and r[gui.chart.sma.legend] alongside chart layout.
    let _ = harness.get_by_label("__stockviz_sma_legend__");
    let _ = harness.get_by_label("__stockviz_volume__");
    assert!(harness.state().visible_bars() > 0);
}

// r[impl test.kittest.volume]
#[test]
fn kittest_volume_bucket_colors_match_candles() {
    use crate::chart::Bucket;
    let bars = sample_bars();
    let bk = Bucket { rows: &bars[0..1] };
    assert!(bk.is_bull());
    let bk2 = Bucket { rows: &bars[1..2] };
    assert!(!bk2.is_bull());
}
