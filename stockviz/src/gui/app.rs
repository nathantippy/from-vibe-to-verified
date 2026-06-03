//! Main chart window: price + volume panes, zoom buttons, anchor semantics.
//! Tag index: [`docs/TALK_TAG_APPENDIX.md`](../../docs/TALK_TAG_APPENDIX.md).

use eframe::egui::{self, Align2, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use tracing::instrument;

use crate::chart;
use crate::data::Bar;

// r[impl gui.chart.candles]
const BULL: Color32 = Color32::from_rgb(0x26, 0xa6, 0x9a);
// r[impl gui.chart.candles]
const BEAR: Color32 = Color32::from_rgb(0xef, 0x53, 0x50);
// r[impl gui.chart.candles]
const SMA_50_COLOR: Color32 = Color32::from_rgb(0x42, 0xa5, 0xf5);
// r[impl gui.chart.candles]
const SMA_150_COLOR: Color32 = Color32::from_rgb(0xff, 0xa7, 0x26);
// r[impl gui.chart.resize]
const MIN_PX_PER_BAR: f32 = 4.0;

// r[impl gui.core]
pub struct StockvizApp {
    bars: Vec<Bar>,
    sma: Vec<Option<f64>>,
    sma150: Vec<Option<f64>>,
    anchor_idx: usize,
    /// User zoom level: count of bars in the visible time span (**`r[gui.chart.zoom]`**).
    visible_bars: usize,
    last_price_w: f32,
}

// r[impl gui.core]
impl StockvizApp {
    // r[impl gui.chart.anchor]
    #[instrument(skip(_cc), fields(n = bars.len()))]
    pub fn new(_cc: &eframe::CreationContext<'_>, bars: Vec<Bar>) -> Self {
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
        let sma = crate::data::sma_50(&closes);
        let sma150 = crate::data::sma_150(&closes);
        let d_cal = crate::data::ny_today();
        let anchor = crate::data::anchor_date(&bars, d_cal)
            .unwrap_or_else(|| bars.last().expect("non-empty").date);
        let anchor_idx =
            crate::data::last_index_on_or_before(&bars, anchor).expect("non-empty chart");
        let max_bars = anchor_idx + 1;
        Self {
            bars,
            sma,
            sma150,
            anchor_idx,
            visible_bars: max_bars,
            last_price_w: 0.0,
        }
    }

    // r[impl gui.chart.anchor]
    fn max_bars(&self) -> usize {
        self.anchor_idx + 1
    }

    // r[impl gui.chart.zoom]
    fn is_full_history_zoom(&self) -> bool {
        self.visible_bars >= self.max_bars()
    }

    // r[impl gui.chart.zoom]
    /// Visible trading-day span (time axis), not pixel fit (**`r[gui.chart.candles]`**).
    fn time_span_bars(&self) -> usize {
        self.visible_bars.min(self.max_bars())
    }

    // r[impl gui.chart.resize]
    fn apply_resize_widen(&mut self, price_w: f32) {
        if price_w > self.last_price_w {
            let max_bars = self.max_bars();
            if self.is_full_history_zoom() {
                if self.visible_bars < max_bars {
                    self.visible_bars = max_bars;
                }
            } else {
                let fit = chart::bars_fitting_width(price_w, MIN_PX_PER_BAR).min(max_bars);
                let current = self.time_span_bars();
                if fit > current {
                    self.visible_bars = fit.min(max_bars);
                }
            }
        }
        self.last_price_w = price_w;
    }

    // r[impl gui.chart.zoom]
    #[cfg(test)]
    pub fn visible_bars(&self) -> usize {
        self.visible_bars
    }

    // r[impl gui.chart.anchor]
    #[cfg(test)]
    pub fn max_bars_pub(&self) -> usize {
        self.max_bars()
    }

    // r[impl gui.chart.zoom]
    #[cfg(test)]
    pub fn time_span_bars_pub(&self) -> usize {
        self.time_span_bars()
    }

    // r[impl gui.chart.anchor]
    #[cfg(test)]
    pub fn visible_start_idx(&self, _price_w: f32) -> usize {
        let n = self.time_span_bars();
        chart::visible_range(self.anchor_idx, n).0
    }

    // r[impl gui.chart.candles]
    fn price_y_range(
        slice: &[Bar],
        start_idx: usize,
        sma: &[Option<f64>],
        sma150: &[Option<f64>],
    ) -> (f64, f64) {
        let mut ymin = f64::INFINITY;
        let mut ymax = f64::NEG_INFINITY;
        for b in slice {
            ymin = ymin.min(b.low);
            ymax = ymax.max(b.high);
        }
        for (i, _) in slice.iter().enumerate() {
            let gi = start_idx + i;
            for series in [sma, sma150] {
                if let Some(v) = series.get(gi).and_then(|x| *x) {
                    ymin = ymin.min(v);
                    ymax = ymax.max(v);
                }
            }
        }
        if !(ymin.is_finite() && ymax.is_finite()) || (ymax - ymin).abs() < 1e-12 {
            ymin -= 1.0;
            ymax += 1.0;
        }
        let pad = (ymax - ymin) * 0.05;
        (ymin - pad, ymax + pad)
    }

    // r[impl gui.chart.candles]
    // r[impl gui.chart.sma.align]
    fn paint_sma_polyline(
        &self,
        painter: &egui::Painter,
        slice: &[Bar],
        start_idx: usize,
        series: &[Option<f64>],
        chart_rect: Rect,
        n_buckets: usize,
        color: Color32,
        y_scale: &dyn Fn(f64) -> f32,
    ) {
        let w = chart_rect.width();
        let mut points = Vec::new();
        for (i, _b) in slice.iter().enumerate() {
            let gi = start_idx + i;
            if let Some(v) = series.get(gi).and_then(|x| *x) {
                let x =
                    chart::sma_screen_x_for_bar(i, slice.len(), chart_rect.left(), w, n_buckets);
                points.push(Pos2::new(x, y_scale(v)));
            }
        }
        if points.len() >= 2 {
            painter.add(egui::Shape::line(points, Stroke::new(1.5_f32, color)));
        }
    }

    // r[impl gui.chart.sma.legend]
    fn paint_sma_legend(&self, painter: &egui::Painter, rect: Rect) {
        let labels = chart::sma_legend_labels();
        let colors = [SMA_50_COLOR, SMA_150_COLOR];
        let font = egui::FontId::monospace(11.0);
        let mut y = rect.top() + 6.0;
        for (label, color) in labels.iter().zip(colors.iter()) {
            let line_y = y + 6.0;
            painter.line_segment(
                [
                    Pos2::new(rect.left() + 6.0, line_y),
                    Pos2::new(rect.left() + 22.0, line_y),
                ],
                Stroke::new(2.0_f32, *color),
            );
            painter.text(
                Pos2::new(rect.left() + 28.0, y),
                Align2::LEFT_TOP,
                *label,
                font.clone(),
                Color32::LIGHT_GRAY,
            );
            y += 18.0;
        }
    }

    // r[impl gui.chart.yticks]
    fn paint_price_yticks(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        ymin: f64,
        ymax: f64,
        y_scale: &dyn Fn(f64) -> f32,
    ) {
        let ticks = chart::price_ticks(ymin, ymax, rect.height().max(1.0) as usize);
        let font = egui::FontId::monospace(10.0);
        for p in ticks {
            let y = y_scale(p);
            painter.text(
                Pos2::new(rect.right() - 4.0, y),
                Align2::RIGHT_CENTER,
                chart::format_price(p),
                font.clone(),
                Color32::GRAY,
            );
        }
    }

    // r[impl gui.chart.candles]
    // r[impl gui.chart.sma.align]
    // r[impl gui.chart.width.fill]
    // r[impl gui.chart.pane.align]
    // r[impl gui.chart.xticks]
    // r[impl gui.chart.yticks]
    // r[impl gui.chart.sma.legend]
    // r[impl gui.chart.pane.align]
    fn chart_drawable_rect(pane_rect: Rect) -> Rect {
        Rect::from_min_max(
            pane_rect.left_top(),
            Pos2::new(
                chart::pane_chart_right(pane_rect.left(), pane_rect.width()),
                pane_rect.bottom(),
            ),
        )
    }

    // r[impl gui.chart.candles]
    // r[impl gui.chart.sma.align]
    // r[impl gui.chart.width.fill]
    // r[impl gui.chart.pane.align]
    // r[impl gui.chart.xticks]
    // r[impl gui.chart.yticks]
    // r[impl gui.chart.sma.legend]
    fn paint_price(&self, painter: &egui::Painter, rect: Rect, slice: &[Bar], start_idx: usize) {
        let chart_rect = Self::chart_drawable_rect(rect);
        let w = chart_rect.width().max(1.0);
        let h = chart_rect.height().max(1.0);
        let width_px = w as usize;
        let buckets = chart::buckets_for_pane(slice, width_px);

        let (ymin, ymax) = Self::price_y_range(slice, start_idx, &self.sma, &self.sma150);

        let y_scale = |price: f64| -> f32 {
            let t = (price - ymin) / (ymax - ymin);
            chart_rect.bottom() - (t as f32) * h
        };

        let n_buckets = buckets.len();
        let col_w = w / n_buckets.max(1) as f32;
        chart::debug_assert_sma_candle_x_alignment(slice.len(), chart_rect.left(), w, n_buckets);

        for (i, bk) in buckets.iter().enumerate() {
            if bk.rows.is_empty() {
                continue;
            }
            let xc = chart::bucket_center_x(chart_rect.left(), w, i, n_buckets);
            let x0 = chart_rect.left() + i as f32 * col_w + 1.0;
            let x1 = chart_rect.left() + (i + 1) as f32 * col_w - 1.0;
            let o = y_scale(bk.synthetic_open());
            let c = y_scale(bk.synthetic_close());
            let hi = y_scale(bk.synthetic_high());
            let lo = y_scale(bk.synthetic_low());
            let top = o.min(c);
            let bot = o.max(c);
            let color = if bk.is_bull() { BULL } else { BEAR };
            painter.line_segment(
                [Pos2::new(xc, hi), Pos2::new(xc, lo)],
                Stroke::new(1.0_f32, color),
            );
            let body = Rect::from_min_max(Pos2::new(x0, top), Pos2::new(x1, bot));
            painter.rect_filled(body, 0.0, color);
        }

        self.paint_sma_polyline(
            painter,
            slice,
            start_idx,
            &self.sma,
            chart_rect,
            n_buckets,
            SMA_50_COLOR,
            &y_scale,
        );
        self.paint_sma_polyline(
            painter,
            slice,
            start_idx,
            &self.sma150,
            chart_rect,
            n_buckets,
            SMA_150_COLOR,
            &y_scale,
        );

        self.paint_price_yticks(painter, rect, ymin, ymax, &y_scale);
        self.paint_sma_legend(painter, chart_rect);

        let n_labels = (w / 120.0).floor().max(1.0) as usize;
        let step = (buckets.len() / n_labels).max(1);
        for i in (0..buckets.len()).step_by(step) {
            if buckets[i].rows.is_empty() {
                continue;
            }
            let d = buckets[i].rows[0].date;
            let x = chart_rect.left() + i as f32 * col_w;
            painter.text(
                Pos2::new(x, chart_rect.bottom() - 14.0),
                Align2::LEFT_TOP,
                d.format("%Y-%m-%d").to_string(),
                egui::FontId::monospace(10.0),
                Color32::GRAY,
            );
        }
    }

    // r[impl gui.chart.volume]
    // r[impl gui.chart.width.fill]
    // r[impl gui.chart.pane.align]
    fn paint_volume(&self, painter: &egui::Painter, rect: Rect, slice: &[Bar]) {
        let chart_rect = Self::chart_drawable_rect(rect);
        let w = chart_rect.width().max(1.0);
        let h = chart_rect.height().max(1.0);
        let width_px = w as usize;
        let buckets = chart::buckets_for_pane(slice, width_px);
        let max_v: f64 = buckets
            .iter()
            .map(|b| b.volume_sum())
            .fold(0.0_f64, f64::max);
        let max_v = if max_v > 0.0 { max_v } else { 1.0 };
        let col_w = w / buckets.len().max(1) as f32;

        for (i, bk) in buckets.iter().enumerate() {
            if bk.rows.is_empty() {
                continue;
            }
            let x0 = chart_rect.left() + i as f32 * col_w + 1.0;
            let x1 = chart_rect.left() + (i + 1) as f32 * col_w - 1.0;
            let v = bk.volume_sum();
            let t = (v / max_v) as f32;
            let y0 = chart_rect.bottom() - t * h;
            let color = if bk.is_bull() { BULL } else { BEAR };
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1, chart_rect.bottom())),
                0.0,
                color,
            );
        }
    }
}

// r[impl gui.core]
impl eframe::App for StockvizApp {
    // r[impl gui.core]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Zoom In (+)").clicked() {
                    // r[impl gui.chart.zoom]
                    self.visible_bars =
                        chart::zoom_in_capped(self.visible_bars, self.max_bars());
                }
                if ui.button("Zoom Out (-)").clicked() {
                    // r[impl gui.chart.zoom]
                    self.visible_bars = chart::zoom_out(self.visible_bars, self.max_bars());
                }
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let avail = ui.available_size();
            let vol_h = (avail.y * 0.22).max(96.0).min(avail.y * 0.45);
            let price_h = (avail.y - vol_h).max(40.0);

            let price_w = avail.x.max(1.0);
            self.apply_resize_widen(price_w);

            let n = self.time_span_bars();
            let (start, _) = chart::visible_range(self.anchor_idx, n);
            let slice = &self.bars[start..=self.anchor_idx];

            ui.allocate_ui_with_layout(
                Vec2::new(avail.x, price_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let (pr, _response) =
                        ui.allocate_exact_size(Vec2::new(avail.x, price_h), Sense::hover());
                    let painter = ui.painter_at(pr);
                    self.paint_price(&painter, pr, slice, start);
                    #[cfg(test)]
                    ui.label("__stockviz_sma_legend__");
                },
            );

            ui.allocate_ui_with_layout(
                Vec2::new(avail.x, vol_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let (vr, _response) =
                        ui.allocate_exact_size(Vec2::new(avail.x, vol_h), Sense::hover());
                    let painter = ui.painter_at(vr);
                    self.paint_volume(&painter, vr, slice);
                    #[cfg(test)]
                    ui.label("__stockviz_volume__");
                },
            );
        });
    }
}
