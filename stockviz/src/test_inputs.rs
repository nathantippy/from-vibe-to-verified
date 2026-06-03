//! Shared `Arbitrary` generators for proptest and structured fuzz (**`r[test.arbitrary.shared]`**).
//!
//! Proptest: `arb::<T>()` via `proptest-arbitrary-interop`. Fuzz: `fuzz_target!(|input: PipelineFuzzInput|)`.
//! Unstructured CSV parser aggression stays on **`csv_parse`** (`&[u8]`).

// r[impl test.arbitrary.shared]
use arbitrary::{Arbitrary, Unstructured};
use chrono::NaiveDate;

use crate::data::Bar;

/// Control tail size (legacy wire format and fuzz corpus).
// r[impl test.arbitrary.shared]
pub const CONTROL_TAIL_LEN: usize = 64;

/// Max payload bytes in [`PipelineFuzzInput`].
// r[impl test.arbitrary.shared]
pub const MAX_PIPELINE_PAYLOAD: usize = 65_536;

// r[impl test.arbitrary.shared]
/// Build `n` ascending valid daily bars (OHLCV passes [`crate::data::validate_bar_rows`]).
pub fn valid_bars(n: usize) -> Vec<Bar> {
    let n = n.clamp(1, 5000);
    let base = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    (0..n)
        .map(|i| {
            let d = base + chrono::Duration::days(i as i64);
            let o = 10.0 + i as f64;
            Bar {
                date: d,
                open: o,
                high: o + 2.0,
                low: o - 1.0,
                close: o + 0.5,
                volume: 100.0 + i as f64,
            }
        })
        .collect()
}

// r[impl test.arbitrary.shared]
/// Encode bars as Twelve Data `time_series` JSON (`{"values":[...]}`).
pub fn encode_time_series_json(bars: &[Bar]) -> Vec<u8> {
    use std::fmt::Write;
    let mut out = String::from(r#"{"values":["#);
    for (i, b) in bars.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write!(
            out,
            r#"{{"datetime":"{}","open":"{}","high":"{}","low":"{}","close":"{}","volume":"{}"}}"#,
            b.date.format("%Y-%m-%d"),
            b.open,
            b.high,
            b.low,
            b.close,
            b.volume
        )
        .unwrap();
    }
    out.push_str("]}");
    out.into_bytes()
}

// r[impl test.arbitrary.shared]
/// Twelve Data JSON body for `r[test.proptest.download.json]` (often > 64 KiB).
#[derive(Debug, Clone)]
pub struct TimeSeriesJsonBody(pub Vec<u8>);

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for TimeSeriesJsonBody {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let n = u.int_in_range(400..=5000)?;
        Ok(Self(encode_time_series_json(&valid_bars(n))))
    }
}

// r[impl test.arbitrary.shared]
/// Valid ascending bar series for chart proptests.
#[derive(Debug, Clone)]
pub struct ValidBarSeries(pub Vec<Bar>);

// r[impl test.arbitrary.shared]
impl ValidBarSeries {
    // r[impl test.arbitrary.shared]
    pub fn bars(&self) -> &[Bar] {
        &self.0
    }

    // r[impl test.arbitrary.shared]
    pub fn into_inner(self) -> Vec<Bar> {
        self.0
    }
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for ValidBarSeries {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let n = u.int_in_range(1..=200)?;
        Ok(ValidBarSeries(valid_bars(n)))
    }
}

// r[impl test.arbitrary.shared]
/// `n` in `2..=40` for CSV round-trip proptest.
#[derive(Debug, Clone)]
pub struct CsvRoundtripSeries(pub ValidBarSeries);

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for CsvRoundtripSeries {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let n = u.int_in_range(2..=40)?;
        Ok(CsvRoundtripSeries(ValidBarSeries(valid_bars(n))))
    }
}

// r[impl test.arbitrary.shared]
/// Layout parameters shared by chart property tests.
#[derive(Debug, Clone, Copy)]
pub struct ChartLayoutParams {
    pub n_bars: usize,
    pub width_px: usize,
    pub full_pane_width: usize,
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for ChartLayoutParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            n_bars: u.int_in_range(1..=200)?,
            width_px: u.int_in_range(1..=200)?,
            full_pane_width: u.int_in_range(100..=1200)?,
        })
    }
}

// r[impl test.arbitrary.shared]
impl ChartLayoutParams {
    // r[impl test.arbitrary.shared]
    pub fn bars(&self) -> ValidBarSeries {
        ValidBarSeries(valid_bars(self.n_bars))
    }
}

// r[impl test.arbitrary.shared]
/// `r[test.proptest.sma.align]` bounds: `n` 1..80, `width` 1..24.
#[derive(Debug, Clone, Copy)]
pub struct AlignLayoutParams {
    pub n_bars: usize,
    pub width_px: usize,
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for AlignLayoutParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            n_bars: u.int_in_range(1..=80)?,
            width_px: u.int_in_range(1..=24)?,
        })
    }
}

// r[impl test.arbitrary.shared]
impl AlignLayoutParams {
    // r[impl test.arbitrary.shared]
    pub fn bars(&self) -> ValidBarSeries {
        ValidBarSeries(valid_bars(self.n_bars))
    }
}

// r[impl test.arbitrary.shared]
/// `r[test.proptest.chart]` bounds.
#[derive(Debug, Clone, Copy)]
pub struct BucketLayoutParams {
    pub n_bars: usize,
    pub width_px: usize,
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for BucketLayoutParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            n_bars: u.int_in_range(1..=100)?,
            width_px: u.int_in_range(1..=32)?,
        })
    }
}

// r[impl test.arbitrary.shared]
impl BucketLayoutParams {
    // r[impl test.arbitrary.shared]
    pub fn bars(&self) -> ValidBarSeries {
        ValidBarSeries(valid_bars(self.n_bars))
    }
}

// r[impl test.arbitrary.shared]
/// `r[test.proptest.chart.width.fill]` bounds.
#[derive(Debug, Clone, Copy)]
pub struct WidthFillLayoutParams {
    pub n_bars: usize,
    pub width_px: usize,
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for WidthFillLayoutParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            n_bars: u.int_in_range(1..=120)?,
            width_px: u.int_in_range(1..=200)?,
        })
    }
}

// r[impl test.arbitrary.shared]
/// `r[test.proptest.pane.align]` bounds.
#[derive(Debug, Clone, Copy)]
pub struct PaneAlignLayoutParams {
    pub n_bars: usize,
    pub full_pane_width: usize,
}

// r[impl test.arbitrary.shared]
/// `r[test.proptest.zoom.limits]` bounds.
#[derive(Debug, Clone, Copy)]
pub struct ZoomSpanParams {
    pub visible_bars: usize,
    pub max_bars: usize,
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for ZoomSpanParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let max_bars = u.int_in_range(1..=500)?;
        let lo = crate::chart::MIN_ZOOM_VISIBLE_BARS.min(max_bars).max(1);
        let visible_bars = u.int_in_range(lo..=max_bars)?;
        Ok(Self {
            visible_bars,
            max_bars,
        })
    }
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for PaneAlignLayoutParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            n_bars: u.int_in_range(2..=200)?,
            full_pane_width: u.int_in_range(100..=1200)?,
        })
    }
}

// r[impl test.arbitrary.shared]
impl PaneAlignLayoutParams {
    // r[impl test.arbitrary.shared]
    pub fn bars(&self) -> ValidBarSeries {
        ValidBarSeries(valid_bars(self.n_bars))
    }
}

// r[impl test.arbitrary.shared]
/// Close prices for 50-period SMA reference proptest.
#[derive(Debug, Clone)]
pub struct Sma50Series {
    pub closes: Vec<f64>,
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for Sma50Series {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(50..=120)?;
        let mut closes = Vec::with_capacity(len);
        for _ in 0..len {
            closes.push(u.int_in_range(-1000..=1000)? as f64 / 1.0);
        }
        Ok(Sma50Series { closes })
    }
}

// r[impl test.arbitrary.shared]
/// Close prices for 150-period SMA reference proptest.
#[derive(Debug, Clone)]
pub struct Sma150Series {
    pub closes: Vec<f64>,
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for Sma150Series {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.int_in_range(150..=250)?;
        let mut closes = Vec::with_capacity(len);
        for _ in 0..len {
            closes.push(u.int_in_range(-1000..=1000)? as f64 / 1.0);
        }
        Ok(Sma150Series { closes })
    }
}

// r[impl test.arbitrary.shared]
/// Which payload interpretation to use in the pipeline harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineMode {
    Csv,
    Json,
    Both,
}

// r[impl test.arbitrary.shared]
impl PipelineMode {
    // r[impl test.arbitrary.shared]
    pub fn from_byte(b: u8) -> Self {
        match b % 3 {
            0 => PipelineMode::Csv,
            1 => PipelineMode::Json,
            _ => PipelineMode::Both,
        }
    }

    // r[impl test.arbitrary.shared]
    pub const fn as_byte(self) -> u8 {
        match self {
            PipelineMode::Csv => 0,
            PipelineMode::Json => 1,
            PipelineMode::Both => 2,
        }
    }
}

// r[impl test.arbitrary.shared]
/// Chart/SMA control bytes (64-byte legacy tail).
#[derive(Debug, Clone, Copy)]
pub struct ChartFuzzCtrl {
    pub width_px: usize,
    pub alt_width_px: usize,
    pub anchor_idx: usize,
    pub n_visible: usize,
    pub left: f32,
    pub width_f: f32,
    pub pane_width: usize,
    pub height_px: usize,
    pub sma_period: usize,
    pub price_w: f32,
    pub ymin: f64,
    pub ymax: f64,
    pub bar_probe: usize,
    pub d_cal_days: i32,
    pub zoom_iters: u8,
    pub opcode: u8,
}

// r[impl test.arbitrary.shared]
impl ChartFuzzCtrl {
    // r[impl test.arbitrary.shared]
    pub fn from_bytes(ctrl: &[u8]) -> Self {
        let mut pad = [0u8; CONTROL_TAIL_LEN];
        let n = ctrl.len().min(CONTROL_TAIL_LEN);
        pad[..n].copy_from_slice(&ctrl[..n]);
        let u32_at = |i: usize| -> u32 {
            let j = i * 4;
            u32::from_le_bytes(pad[j..j + 4].try_into().unwrap())
        };
        let f32_at = |i: usize| -> f32 { f32::from_bits(u32_at(i)) };
        Self {
            width_px: (u32_at(0) as usize) % 8193,
            alt_width_px: (u32_at(1) as usize) % 8193,
            anchor_idx: (u32_at(2) as usize) % 10_000,
            n_visible: ((u32_at(3) as usize) % 10_000).max(1),
            pane_width: (u32_at(4) as usize) % 8193,
            height_px: ((u32_at(5) as usize) % 4096).max(1),
            sma_period: (u32_at(6) as usize) % 512,
            bar_probe: (u32_at(7) as usize) % 10_000,
            left: f32_at(8),
            width_f: f32_at(9).abs().max(1.0),
            price_w: f32_at(10).abs(),
            ymin: f64::from_bits(u64::from_le_bytes(pad[44..52].try_into().unwrap())),
            ymax: f64::from_bits(u64::from_le_bytes(pad[52..60].try_into().unwrap())),
            d_cal_days: i32::from_le_bytes(pad[60..64].try_into().unwrap()),
            zoom_iters: pad[12],
            opcode: pad[13],
        }
    }

    // r[impl test.arbitrary.shared]
    pub fn to_bytes(self) -> [u8; CONTROL_TAIL_LEN] {
        let mut pad = [0u8; CONTROL_TAIL_LEN];
        let mut write_u32 = |i: usize, v: u32| {
            pad[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        };
        write_u32(0, self.width_px as u32);
        write_u32(1, self.alt_width_px as u32);
        write_u32(2, self.anchor_idx as u32);
        write_u32(3, self.n_visible as u32);
        write_u32(4, self.pane_width as u32);
        write_u32(5, self.height_px as u32);
        write_u32(6, self.sma_period as u32);
        write_u32(7, self.bar_probe as u32);
        write_u32(8, self.left.to_bits());
        write_u32(9, self.width_f.to_bits());
        write_u32(10, self.price_w.to_bits());
        pad[44..52].copy_from_slice(&self.ymin.to_bits().to_le_bytes());
        pad[52..60].copy_from_slice(&self.ymax.to_bits().to_le_bytes());
        pad[60..64].copy_from_slice(&self.d_cal_days.to_le_bytes());
        pad[12] = self.zoom_iters;
        pad[13] = self.opcode;
        pad
    }
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for ChartFuzzCtrl {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut pad = [0u8; CONTROL_TAIL_LEN];
        u.fill_buffer(&mut pad)?;
        Ok(ChartFuzzCtrl::from_bytes(&pad))
    }
}

// r[impl test.arbitrary.shared]
/// Structured fuzz / pipeline input (`[mode][payload][64-byte ctrl]` wire format).
#[derive(Debug, Clone)]
pub struct PipelineFuzzInput {
    pub mode: PipelineMode,
    pub payload: Vec<u8>,
    pub ctrl: ChartFuzzCtrl,
}

// r[impl test.arbitrary.shared]
impl PipelineFuzzInput {
    // r[impl test.arbitrary.shared]
    pub fn encode_legacy(&self) -> Vec<u8> {
        let mut data = vec![self.mode.as_byte()];
        data.extend_from_slice(&self.payload);
        data.extend_from_slice(&self.ctrl.to_bytes());
        data
    }

    // r[impl test.arbitrary.shared]
    pub fn from_legacy_bytes(data: &[u8]) -> arbitrary::Result<Self> {
        if data.is_empty() {
            return Ok(Self {
                mode: PipelineMode::Csv,
                payload: Vec::new(),
                ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
            });
        }
        let mode = PipelineMode::from_byte(data[0]);
        let tail = CONTROL_TAIL_LEN.min(data.len().saturating_sub(1));
        let payload_end = data.len().saturating_sub(tail);
        let mut payload = if payload_end > 1 {
            data[1..payload_end].to_vec()
        } else {
            Vec::new()
        };
        if payload.len() > MAX_PIPELINE_PAYLOAD {
            payload.truncate(MAX_PIPELINE_PAYLOAD);
        }
        let ctrl_bytes = if tail > 0 {
            &data[payload_end..]
        } else {
            &[] as &[u8]
        };
        Ok(Self {
            mode,
            payload,
            ctrl: ChartFuzzCtrl::from_bytes(ctrl_bytes),
        })
    }

    /// Decode corpus / fuzz bytes (legacy layout).
    // r[impl test.arbitrary.shared]
    pub fn from_bytes(data: &[u8]) -> arbitrary::Result<Self> {
        Self::from_legacy_bytes(data)
    }

    // r[impl test.arbitrary.shared]
    pub fn exercise(&self) {
        crate::fuzz_harness::exercise_pipeline_input(self);
    }
}

// r[impl test.arbitrary.shared]
impl<'a> Arbitrary<'a> for PipelineFuzzInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.is_empty() {
            return Ok(PipelineFuzzInput {
                mode: PipelineMode::Csv,
                payload: Vec::new(),
                ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
            });
        }
        let len = u.len();
        let mut buf = vec![0u8; len];
        u.fill_buffer(&mut buf)?;
        let mut input = Self::from_legacy_bytes(&buf)?;
        if input.payload.len() > MAX_PIPELINE_PAYLOAD {
            input.payload.truncate(MAX_PIPELINE_PAYLOAD);
        }
        Ok(input)
    }
}

// r[impl test.arbitrary.shared]
#[cfg(test)]
mod tests {
    // r[impl test.arbitrary.shared]
    use super::*;
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    // r[impl test.arbitrary.shared]
    // r[verify test.arbitrary.proptest]
    #[test]
    fn legacy_roundtrip_encoding() {
        let input = PipelineFuzzInput {
            mode: PipelineMode::Both,
            payload: b"hello".to_vec(),
            ctrl: ChartFuzzCtrl::from_bytes(&[0xAB; CONTROL_TAIL_LEN]),
        };
        let bytes = input.encode_legacy();
        let back = PipelineFuzzInput::from_legacy_bytes(&bytes).unwrap();
        assert_eq!(back.mode, input.mode);
        assert_eq!(back.payload, input.payload);
    }

    // r[impl test.arbitrary.proptest]
    // r[verify test.arbitrary.proptest]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(8))]
        #[test]
        fn pipeline_input_smoke(input in arb::<PipelineFuzzInput>()) {
            input.exercise();
        }
    }

    // r[impl test.arbitrary.shared]
    /// Writes `fuzz/corpus/my_target/seed_*`. Run: `./scripts/seed_pipeline_corpus.sh`
    #[test]
    #[ignore]
    fn write_pipeline_corpus_seeds() {
        use std::fs;
        use std::path::PathBuf;

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/my_target");
        fs::create_dir_all(&dir).expect("corpus dir");

        const CORPUS_README: &str = r"# `my_target` corpus

Structured inputs use [`PipelineFuzzInput`](../../../src/test_inputs.rs) (`Arbitrary` / legacy wire format).

**Wire format:** `[mode u8][payload…][64-byte ChartFuzzCtrl]`

| `mode` | Meaning |
|--------|---------|
| `0` | CSV payload only |
| `1` | Twelve Data JSON payload only |
| `2` | Both |

**Regenerate committed seeds:** `./scripts/seed_pipeline_corpus.sh`

Only `seed_*` files and this README are tracked in git. LibFuzzer hash shards in this directory are gitignored.
";
        fs::write(dir.join("README.md"), CORPUS_README).expect("write corpus README");

        let valid_csv = b"Date,Open,High,Low,Close,Volume\n\
2020-01-01,10,12,9,11,100\n\
2020-01-02,11,11,10,10,200\n";

        let json = br#"{"values":[
{"datetime":"2020-01-02","open":"10","high":"11","low":"9","close":"10.5","volume":"100"},
{"datetime":"2020-01-01","open":"9","high":"10","low":"8","close":"10","volume":"50"}
]}"#;

        let seeds: [(&str, PipelineFuzzInput); 7] = [
            (
                "seed_valid_csv_mode0",
                PipelineFuzzInput {
                    mode: PipelineMode::Csv,
                    payload: valid_csv.to_vec(),
                    ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
                },
            ),
            (
                "seed_json_mode1",
                PipelineFuzzInput {
                    mode: PipelineMode::Json,
                    payload: json.to_vec(),
                    ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
                },
            ),
            (
                "seed_dual_mode2",
                PipelineFuzzInput {
                    mode: PipelineMode::Both,
                    payload: valid_csv.to_vec(),
                    ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
                },
            ),
            (
                "seed_non_finite_csv",
                PipelineFuzzInput {
                    mode: PipelineMode::Csv,
                    payload: b"Date,Open,High,Low,Close,Volume\n2020-01-01,inf,12,9,11,100\n".to_vec(),
                    ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
                },
            ),
            (
                "seed_bad_header",
                PipelineFuzzInput {
                    mode: PipelineMode::Csv,
                    payload: std::fs::read(dir.parent().unwrap().join("csv_parse/bad_header.csv"))
                        .unwrap_or_else(|_| b"bad".to_vec()),
                    ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
                },
            ),
            (
                "seed_control_only",
                PipelineFuzzInput {
                    mode: PipelineMode::Csv,
                    payload: Vec::new(),
                    ctrl: ChartFuzzCtrl::from_bytes(&[0u8; CONTROL_TAIL_LEN]),
                },
            ),
            (
                "seed_wild_ctrl",
                PipelineFuzzInput {
                    mode: PipelineMode::Csv,
                    payload: valid_csv.to_vec(),
                    ctrl: ChartFuzzCtrl::from_bytes(&[0xFF; CONTROL_TAIL_LEN]),
                },
            ),
        ];

        for (name, input) in seeds {
            let path = dir.join(name);
            fs::write(&path, input.encode_legacy()).expect("write seed");
        }
    }
}
