//! Pipeline exercise for CSV / JSON → chart math (**`r[test.fuzz.pipeline]`**).
//!
//! Structured input: [`crate::test_inputs::PipelineFuzzInput`] (`Arbitrary` + legacy bytes).
//! See [`docs/ARBITRARY_TESTING.md`](../docs/ARBITRARY_TESTING.md).

// r[impl test.fuzz.pipeline]
use chrono::NaiveDate;

use crate::chart::{
    bars_fitting_width, bucket_center_x, bucket_index_for_bar, buckets_for_pane, buckets_for_width,
    format_price, last_bar_center_x, layout_column_count, pane_chart_right, pane_chart_width_px,
    price_ticks, sma_legend_labels, sma_screen_x_for_bar, visible_range, zoom_in_capped, zoom_out,
};
use crate::data::{
    anchor_date, last_index_on_or_before, parse_csv_bytes, sma, sma_150, sma_50, write_csv, Bar,
};
use crate::test_inputs::{
    ChartFuzzCtrl, PipelineFuzzInput, PipelineMode, CONTROL_TAIL_LEN, MAX_PIPELINE_PAYLOAD,
};

const MAX_BAR_PROBE: usize = 256;
const MAX_ROUNDTRIP_ROWS: usize = 5000;

// r[impl test.fuzz.pipeline]
pub use crate::test_inputs::CONTROL_TAIL_LEN as FUZZ_CONTROL_TAIL_LEN;

// r[impl test.fuzz.pipeline]
/// Split legacy fuzz bytes into mode, payload, and control tail.
pub fn split_fuzz_input(data: &[u8]) -> (u8, &[u8], &[u8]) {
    if let Ok(input) = PipelineFuzzInput::from_legacy_bytes(data) {
        let mode = input.mode.as_byte();
        let tail_len = CONTROL_TAIL_LEN.min(data.len().saturating_sub(1));
        let payload_end = data.len().saturating_sub(tail_len);
        let payload = if payload_end > 1 {
            &data[1..payload_end]
        } else {
            &[]
        };
        let ctrl = if tail_len > 0 {
            &data[payload_end..]
        } else {
            &[]
        };
        (mode, payload, ctrl)
    } else {
        (0, &[], &[])
    }
}

// r[impl test.fuzz.pipeline]
fn exercise_zoom_chain(mut vis: usize, max_bars: usize, iters: u8) {
    let n = (iters & 7).max(1);
    for _ in 0..n {
        vis = zoom_in_capped(vis, max_bars.max(1));
        vis = zoom_out(vis, max_bars.max(1));
    }
    let _ = vis;
}

// r[impl test.fuzz.pipeline]
pub fn exercise_chart_params_only(ctrl: &ChartFuzzCtrl) {
    let n_bars = (ctrl.bar_probe % 500).max(1);
    let cols = layout_column_count(n_bars, ctrl.pane_width.max(ctrl.width_px));
    let _ = sma_legend_labels();
    let _ = buckets_for_width(&[], ctrl.width_px);
    let _ = buckets_for_pane(&[], ctrl.alt_width_px);
    let bi = bucket_index_for_bar(ctrl.bar_probe, n_bars, cols.max(1));
    let _ = bucket_center_x(ctrl.left, ctrl.width_f, bi, cols.max(1));
    let _ = sma_screen_x_for_bar(ctrl.bar_probe % n_bars, n_bars, ctrl.left, ctrl.width_f, cols);
    let _ = last_bar_center_x(n_bars, ctrl.left, ctrl.width_f, ctrl.pane_width);
    let anchor = ctrl.anchor_idx % n_bars.max(1);
    let (start, nv) = visible_range(anchor, ctrl.n_visible);
    let _ = visible_range(n_bars.saturating_add(anchor), ctrl.n_visible);
    let _ = (start, nv);
    exercise_zoom_chain(ctrl.n_visible, n_bars, ctrl.zoom_iters);
    let _ = bars_fitting_width(ctrl.price_w, 4.0);
    let _ = pane_chart_width_px(ctrl.width_f);
    let _ = pane_chart_right(ctrl.left, ctrl.width_f);
    let _ = price_ticks(ctrl.ymin, ctrl.ymax, ctrl.height_px);
    let _ = format_price(ctrl.ymin);
    let _ = sma(&[], ctrl.sma_period);
}

// r[impl test.fuzz.pipeline]
fn exercise_bar_indices(bars: &[Bar], ctrl: &ChartFuzzCtrl) {
    let n = bars.len();
    if n == 0 {
        return;
    }
    let cols = layout_column_count(n, ctrl.pane_width.max(ctrl.width_px).max(ctrl.alt_width_px));
    let cols = cols.max(1);
    let probe = n.min(MAX_BAR_PROBE);
    for i in 0..probe {
        let bi = bucket_index_for_bar(i, n, cols);
        let x = sma_screen_x_for_bar(i, n, ctrl.left, ctrl.width_f, cols);
        let xc = bucket_center_x(ctrl.left, ctrl.width_f, bi, cols);
        let _ = (x, xc);
    }
    if n > MAX_BAR_PROBE {
        let i = ctrl.bar_probe % n;
        let bi = bucket_index_for_bar(i, n, cols);
        let _ = bucket_center_x(ctrl.left, ctrl.width_f, bi, cols);
    }
}

// r[impl test.fuzz.pipeline]
pub fn exercise_chart_on_bars(bars: &[Bar], ctrl: &ChartFuzzCtrl) {
    if bars.is_empty() || bars.len() > MAX_ROUNDTRIP_ROWS {
        return;
    }

    let n = bars.len();
    let anchor = ctrl.anchor_idx % n;
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let _ = sma_50(&closes);
    let _ = sma_150(&closes);
    let _ = sma(&closes, ctrl.sma_period.max(1));

    for width in [ctrl.width_px, ctrl.alt_width_px] {
        let buckets = buckets_for_pane(bars, width);
        for b in &buckets {
            let _ = (
                b.synthetic_open(),
                b.synthetic_high(),
                b.synthetic_low(),
                b.synthetic_close(),
                b.volume_sum(),
                b.is_bull(),
            );
        }
        let _ = buckets_for_width(bars, width);
    }

    let (start, nv) = visible_range(anchor, ctrl.n_visible);
    let end = (start + nv).min(n);
    let window = &bars[start..end];
    if !window.is_empty() {
        let ymin = window.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let ymax = window.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let _ = price_ticks(ymin, ymax, ctrl.height_px);
    }

    exercise_bar_indices(bars, ctrl);
    let _ = last_bar_center_x(n, ctrl.left, ctrl.width_f, ctrl.pane_width);
    exercise_zoom_chain(nv, n, ctrl.zoom_iters);

    if let Some(last) = bars.last() {
        let day = last.date + chrono::Duration::days(i64::from(ctrl.d_cal_days.unsigned_abs() % 10_000));
        let _ = last_index_on_or_before(bars, day);
        let _ = anchor_date(bars, day);
        let _ = anchor_date(bars, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    }
}

// r[impl test.fuzz.pipeline]
pub fn exercise_csv_roundtrip(bars: &[Bar]) {
    if bars.is_empty() || bars.len() > MAX_ROUNDTRIP_ROWS {
        return;
    }
    let file = match tempfile::NamedTempFile::new() {
        Ok(f) => f,
        Err(_) => return,
    };
    if write_csv(file.path(), bars).is_err() {
        return;
    }
    let bytes = match std::fs::read(file.path()) {
        Ok(b) => b,
        Err(_) => return,
    };
    if let Ok(round) = parse_csv_bytes(&bytes) {
        exercise_chart_on_bars(&round, &ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]));
    }
}

// r[impl test.fuzz.pipeline]
fn exercise_payload(mode: PipelineMode, payload: &[u8], ctrl: &ChartFuzzCtrl) {
    let try_csv = matches!(mode, PipelineMode::Csv | PipelineMode::Both);
    let try_json = matches!(mode, PipelineMode::Json | PipelineMode::Both);
    if try_csv {
        if let Ok(bars) = parse_csv_bytes(payload) {
            exercise_chart_on_bars(&bars, ctrl);
            exercise_csv_roundtrip(&bars);
        }
    }
    if try_json {
        #[cfg(feature = "twelve-data")]
        if let Ok(bars) = crate::download::twelve_data::parse_time_series_json(payload) {
            exercise_chart_on_bars(&bars, ctrl);
        }
    }
}

// r[impl test.fuzz.pipeline]
fn exercise_csv_aggressor(data: &[u8]) {
    let len = data.len().min(MAX_PIPELINE_PAYLOAD);
    if len == 0 {
        return;
    }
    let _ = parse_csv_bytes(&data[..len]);
}

// r[impl test.fuzz.pipeline]
/// Primary entry: structured [`PipelineFuzzInput`] from proptest or libFuzzer.
pub fn exercise_pipeline_input(input: &PipelineFuzzInput) {
    let ctrl = &input.ctrl;
    exercise_chart_params_only(ctrl);
    exercise_payload(input.mode, &input.payload, ctrl);
    exercise_csv_aggressor(&input.encode_legacy());

    if ctrl.opcode & 1 != 0 {
        let alt = ChartFuzzCtrl {
            width_px: ctrl.alt_width_px,
            ..*ctrl
        };
        exercise_chart_params_only(&alt);
    }
}

// r[impl test.fuzz.pipeline]
/// Legacy byte corpus (`[mode][payload][ctrl]`).
pub fn exercise_fuzz_input(data: &[u8]) {
    if let Ok(input) = PipelineFuzzInput::from_legacy_bytes(data) {
        exercise_pipeline_input(&input);
    } else {
        exercise_chart_params_only(&ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]));
        exercise_csv_aggressor(data);
    }
}

// r[impl test.fuzz.pipeline]
#[cfg(test)]
mod tests {
    // r[impl test.fuzz.pipeline]
    use super::*;
    use crate::test_inputs::PipelineMode;

    const VALID_CSV: &str = "Date,Open,High,Low,Close,Volume\n\
2020-01-01,10,12,9,11,100\n\
2020-01-02,11,11,10,10,200\n";

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn split_puts_mode_payload_and_tail() {
        let input = PipelineFuzzInput {
            mode: PipelineMode::Both,
            payload: b"payload".to_vec(),
            ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
        };
        let data = input.encode_legacy();
        let (mode, payload, ctrl) = split_fuzz_input(&data);
        assert_eq!(mode, 2);
        assert_eq!(payload, b"payload");
        assert_eq!(ctrl.len(), CONTROL_TAIL_LEN);
        let back = PipelineFuzzInput::from_legacy_bytes(&data).unwrap();
        assert_eq!(back.mode, input.mode);
        assert_eq!(back.payload, input.payload);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn valid_csv_mode0_does_not_panic() {
        let input = PipelineFuzzInput {
            mode: PipelineMode::Csv,
            payload: VALID_CSV.as_bytes().to_vec(),
            ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
        };
        exercise_pipeline_input(&input);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn csv_roundtrip_oracle() {
        let bars = parse_csv_bytes(VALID_CSV.as_bytes()).unwrap();
        exercise_csv_roundtrip(&bars);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn malformed_payload_still_runs_chart_params() {
        let input = PipelineFuzzInput {
            mode: PipelineMode::Csv,
            payload: b"not,csv".to_vec(),
            ctrl: ChartFuzzCtrl::from_bytes(&[0xFF; CONTROL_TAIL_LEN]),
        };
        exercise_pipeline_input(&input);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn control_only_tail_does_not_panic() {
        exercise_fuzz_input(&[0u8; 1 + CONTROL_TAIL_LEN]);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn oom_artifact_exercise_does_not_hang() {
        let data =
            include_bytes!("../fuzz/artifacts/my_target/oom-7fc036367c008997f59b0adc14d213e7423b8b29");
        let input = PipelineFuzzInput::from_legacy_bytes(data).unwrap();
        exercise_pipeline_input(&input);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn oversized_payload_does_not_panic() {
        let mut data = vec![1u8]; // Json mode
        data.extend(std::iter::repeat_n(b'x', MAX_PIPELINE_PAYLOAD + 1024));
        data.extend([0u8; CONTROL_TAIL_LEN]);
        let input = PipelineFuzzInput::from_legacy_bytes(&data).unwrap();
        assert!(input.payload.len() <= MAX_PIPELINE_PAYLOAD);
        exercise_pipeline_input(&input);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[cfg(feature = "twelve-data")]
    #[test]
    fn json_mode1_sample_body() {
        let body = br#"{"values":[
            {"datetime":"2020-01-02","open":"10","high":"11","low":"9","close":"10.5","volume":"100"},
            {"datetime":"2020-01-01","open":"9","high":"10","low":"8","close":"10","volume":"50"}
        ]}"#;
        let input = PipelineFuzzInput {
            mode: PipelineMode::Json,
            payload: body.to_vec(),
            ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
        };
        exercise_pipeline_input(&input);
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn many_row_csv_under_roundtrip_cap() {
        let input = PipelineFuzzInput {
            mode: PipelineMode::Csv,
            payload: {
                let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
                let mut s = String::from("Date,Open,High,Low,Close,Volume\n");
                for i in 0..200 {
                    let d = start + chrono::Duration::days(i);
                    s.push_str(&format!("{d},10,12,9,11,100\n"));
                }
                s.into_bytes()
            },
            ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
        };
        exercise_pipeline_input(&input);
    }
}
