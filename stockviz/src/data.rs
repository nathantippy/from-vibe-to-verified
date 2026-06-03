use std::io::Read;
use std::path::Path;

use chrono::NaiveDate;
use chrono_tz::America::New_York;
use tracing::instrument;

use crate::error::{Error, Result};

// r[impl data.format]
const HEADER: &str = "Date,Open,High,Low,Close,Volume";

/// One CSV row (daily bar).
// r[impl data.format]
#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// r[impl cli.graph.path]
/// Resolve `graph` CLI path: extensionless → `.csv`; `.csv` unchanged (**`r[cli.graph.path]`**).
pub fn resolve_graph_csv_path(path: &Path) -> Result<std::path::PathBuf> {
    match path.extension().and_then(|e| e.to_str()) {
        None => Ok(path.with_extension("csv")),
        Some("csv") => Ok(path.to_path_buf()),
        Some(ext) => Err(Error::Validation(format!(
            "graph path must be .csv or extensionless, not .{ext}"
        ))),
    }
}

/// Parse and validate CSV per **`r[data.format]`** / **`r[data.validation]`**.
// r[impl data.format]
#[instrument(skip(path), fields(path = %path.display()))]
pub fn load_csv(path: &Path) -> Result<Vec<Bar>> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    parse_csv_str(&buf)
}

// r[impl data.format]
// r[impl data.validation]
// r[impl test.fuzz.csv]
#[instrument(skip(data), fields(len = data.len()))]
pub fn parse_csv_bytes(data: &[u8]) -> Result<Vec<Bar>> {
    let mut r = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(data);

    let headers = r.headers()?.clone();
    let expect = csv::ByteRecord::from(vec!["Date", "Open", "High", "Low", "Close", "Volume"]);
    if *headers.as_byte_record() != expect {
        return Err(Error::Validation(format!(
            "CSV header must be exactly `{HEADER}`"
        )));
    }

    let mut rows = Vec::new();
    for rec in r.records() {
        let rec = rec?;
        if rec.len() != 6 {
            return Err(Error::Validation(format!(
                "row has {} columns, expected 6",
                rec.len()
            )));
        }
        let date = NaiveDate::parse_from_str(rec.get(0).unwrap(), "%Y-%m-%d")
            .map_err(|e| Error::DateParse(e.to_string()))?;
        let open: f64 = rec
            .get(1)
            .unwrap()
            .parse()
            .map_err(|_| Error::Validation("Open not a number".into()))?;
        let high: f64 = rec
            .get(2)
            .unwrap()
            .parse()
            .map_err(|_| Error::Validation("High not a number".into()))?;
        let low: f64 = rec
            .get(3)
            .unwrap()
            .parse()
            .map_err(|_| Error::Validation("Low not a number".into()))?;
        let close: f64 = rec
            .get(4)
            .unwrap()
            .parse()
            .map_err(|_| Error::Validation("Close not a number".into()))?;
        let volume: f64 = rec
            .get(5)
            .unwrap()
            .parse()
            .map_err(|_| Error::Validation("Volume not a number".into()))?;

        rows.push(Bar {
            date,
            open,
            high,
            low,
            close,
            volume,
        });
    }

    validate_order(&rows)?;
    validate_bar_rows(&rows)?;
    Ok(rows)
}

/// Parse UTF-8 CSV text (delegates to [`parse_csv_bytes`]).
// r[impl data.format]
// r[impl data.validation]
#[instrument(skip(s), fields(len = s.len()))]
pub fn parse_csv_str(s: &str) -> Result<Vec<Bar>> {
    parse_csv_bytes(s.as_bytes())
}

/// OHLC / volume rules per row (already ordered).
// r[impl data.validation]
pub(crate) fn validate_bar_rows(rows: &[Bar]) -> Result<()> {
    for b in rows {
        if !b.open.is_finite()
            || !b.high.is_finite()
            || !b.low.is_finite()
            || !b.close.is_finite()
            || !b.volume.is_finite()
        {
            return Err(Error::Validation("non-finite OHLCV".into()));
        }
        if b.volume < 0.0 {
            return Err(Error::Validation("Volume must be >= 0".into()));
        }
        let mx = b.open.max(b.close);
        let mn = b.open.min(b.close);
        if b.high + 1e-9 < mx || b.low - 1e-9 > mn {
            return Err(Error::Validation(format!(
                "OHLC inconsistent on {}: H={} L={} O={} C={}",
                b.date, b.high, b.low, b.open, b.close
            )));
        }
    }
    Ok(())
}

// r[impl data.validation]
pub(crate) fn validate_order(rows: &[Bar]) -> Result<()> {
    for w in rows.windows(2) {
        if w[0].date >= w[1].date {
            return Err(Error::Validation(
                "Date must be strictly ascending with no duplicates".into(),
            ));
        }
    }
    Ok(())
}

/// Write CSV in canonical form (oldest first).
// r[impl data.format]
pub fn write_csv(path: &Path, bars: &[Bar]) -> Result<()> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record(["Date", "Open", "High", "Low", "Close", "Volume"])?;
    for b in bars {
        w.write_record([
            b.date.format("%Y-%m-%d").to_string(),
            format!("{}", b.open),
            format!("{}", b.high),
            format!("{}", b.low),
            format!("{}", b.close),
            format!("{}", b.volume),
        ])?;
    }
    w.flush()?;
    Ok(())
}

/// Rolling simple moving average of `close` over `period` rows.
// r[impl data.sma]
// r[impl data.sma150]
pub fn sma(close: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = close.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let mut sum: f64 = close[..period].iter().sum();
    out[period - 1] = Some(sum / period as f64);
    for i in period..n {
        sum += close[i];
        sum -= close[i - period];
        out[i] = Some(sum / period as f64);
    }
    out
}

/// Simple moving average of close over 50 rows; value at index i exists iff i >= 49.
// r[impl data.sma]
pub fn sma_50(close: &[f64]) -> Vec<Option<f64>> {
    sma(close, 50)
}

/// Simple moving average of close over 150 rows; value at index i exists iff i >= 149.
// r[impl data.sma150]
pub fn sma_150(close: &[f64]) -> Vec<Option<f64>> {
    sma(close, 150)
}

/// Calendar "today" in America/New_York as a naive date (local calendar date).
// r[impl gui.chart.timeaxis]
pub fn ny_today() -> NaiveDate {
    chrono::Utc::now().with_timezone(&New_York).date_naive()
}

/// Last index with `bar.date <= anchor`, or None if empty.
// r[impl gui.chart.anchor]
pub fn last_index_on_or_before(bars: &[Bar], anchor: NaiveDate) -> Option<usize> {
    let mut best = None;
    for (i, b) in bars.iter().enumerate() {
        if b.date <= anchor {
            best = Some(i);
        }
    }
    best
}

/// `D_anchor = min(D_cal, D_last)` where D_last is last bar in series.
// r[impl gui.chart.anchor]
pub fn anchor_date(bars: &[Bar], d_cal: NaiveDate) -> Option<NaiveDate> {
    let last = bars.last()?;
    Some(d_cal.min(last.date))
}

// r[impl data.validation]
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // r[impl cli.graph.path]
    // r[verify cli.graph.path]
    #[test]
    fn resolve_graph_appends_csv_extension() {
        let p = resolve_graph_csv_path(std::path::Path::new("NOW")).unwrap();
        assert_eq!(p, PathBuf::from("NOW.csv"));
    }

    // r[impl cli.graph.path]
    // r[verify cli.graph.path]
    #[test]
    fn resolve_graph_keeps_csv_path() {
        let p = resolve_graph_csv_path(std::path::Path::new("NOW.csv")).unwrap();
        assert_eq!(p, PathBuf::from("NOW.csv"));
    }

    // r[impl cli.graph.path]
    // r[verify cli.graph.path]
    #[test]
    fn resolve_graph_rejects_other_extension() {
        assert!(resolve_graph_csv_path(std::path::Path::new("data.json")).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn rejects_unsorted() {
        let s = "Date,Open,High,Low,Close,Volume\n\
2020-01-02,1,1,1,1,10\n\
2020-01-01,1,1,1,1,10\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl data.validation]
    #[test]
    fn rejects_negative_volume() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,1,1,1,1,-1\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl data.format]
    // r[impl data.sma]
    #[test]
    // r[verify data.format]
    // r[verify data.sma]
    fn golden_roundtrip() {
        let s = "Date,Open,High,Low,Close,Volume\n\
2020-01-01,10,12,9,11,100\n\
2020-01-02,11,11,10,10,200\n";
        let rows = parse_csv_str(s).unwrap();
        let sma = sma_50(&rows.iter().map(|b| b.close).collect::<Vec<_>>());
        assert!(sma[0].is_none());

        let closes: Vec<f64> = (0u64..60).map(|i| 100.0 + (i as f64) * 0.1).collect();
        let sm = sma_50(&closes);
        assert!(sm[48].is_none());
        assert!(sm[49].is_some());
    }

    // r[impl data.sma]
    // r[verify data.sma]
    #[test]
    fn sma_50_exactly_fifty_closes() {
        let closes: Vec<f64> = vec![1.0; 50];
        let sm = sma_50(&closes);
        assert!(sm[48].is_none());
        assert!(sm[49].is_some());
        assert!((sm[49].unwrap() - 1.0).abs() < 1e-9);
    }

    // r[impl data.sma150]
    // r[verify data.sma150]
    #[test]
    fn sma_150_exactly_one_fifty_closes() {
        let closes: Vec<f64> = vec![2.0; 150];
        let sm = sma_150(&closes);
        assert!(sm[148].is_none());
        assert!(sm[149].is_some());
        assert!((sm[149].unwrap() - 2.0).abs() < 1e-9);
    }

    // r[impl data.sma150]
    // r[verify data.sma150]
    #[test]
    fn sma_150_none_before_warmup() {
        let closes: Vec<f64> = (0..149).map(|i| i as f64).collect();
        let sm = sma_150(&closes);
        assert!(sm.iter().all(|v| v.is_none()));
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_accepts_zero_volume() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,10,12,9,11,0\n";
        assert_eq!(parse_csv_str(s).unwrap().len(), 1);
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_nan_in_open_only() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,nan,1,1,1,10\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_nan_in_each_ohlcv_field() {
        for bad in [
            "Date,Open,High,Low,Close,Volume\n2020-01-01,nan,12,9,11,10\n",
            "Date,Open,High,Low,Close,Volume\n2020-01-01,10,nan,9,11,10\n",
            "Date,Open,High,Low,Close,Volume\n2020-01-01,10,12,nan,11,10\n",
            "Date,Open,High,Low,Close,Volume\n2020-01-01,10,12,9,nan,10\n",
            "Date,Open,High,Low,Close,Volume\n2020-01-01,10,12,9,11,nan\n",
        ] {
            assert!(parse_csv_str(bad).is_err(), "expected err for {bad}");
        }
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_high_too_low_only() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,10,9.9,8,10,100\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_low_too_high_only() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,10,12,10.0001,10,100\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_accepts_high_at_mx_ceiling() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,10,9.999999999,9,10,100\n";
        assert_eq!(parse_csv_str(s).unwrap().len(), 1);
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_accepts_low_at_mn_floor() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,10,12,10.000000001,10,100\n";
        assert_eq!(parse_csv_str(s).unwrap().len(), 1);
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_ohlc_inconsistent() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,10,9,8,11,100\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl test.fuzz.csv]
    // r[verify test.fuzz.csv]
    #[test]
    fn parse_csv_bytes_rejects_garbage_without_panicking() {
        for raw in [
            b"\xff\xfe\x00" as &[u8],
            b"Date,Open,High,Low,Close,Volume\nx",
            &[0u8; 2048],
            b"not,a,csv,at,all",
        ] {
            let _ = parse_csv_bytes(raw);
        }
    }

    // r[impl data.format]
    // r[verify data.format]
    #[test]
    fn parse_rejects_bad_header() {
        let s = "Date,Open,High,Low,Close,Vol\n2020-01-01,1,1,1,1,10\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl data.format]
    // r[verify data.format]
    #[test]
    fn parse_rejects_wrong_column_count() {
        let s = "Date,Open,High,Low,Close,Volume\n2020-01-01,1,1,1,1\n";
        assert!(parse_csv_str(s).is_err());
    }

    // r[impl data.validation]
    fn sample_bar() -> Bar {
        Bar {
            date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            open: 10.,
            high: 12.,
            low: 9.,
            close: 11.,
            volume: 100.,
        }
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_accepts_zero_volume() {
        let mut b = sample_bar();
        b.volume = 0.;
        assert!(validate_bar_rows(&[b]).is_ok());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_rejects_nan_per_field() {
        let mut o = sample_bar();
        o.open = f64::NAN;
        assert!(validate_bar_rows(&[o]).is_err());
        let mut h = sample_bar();
        h.high = f64::NAN;
        assert!(validate_bar_rows(&[h]).is_err());
        let mut l = sample_bar();
        l.low = f64::NAN;
        assert!(validate_bar_rows(&[l]).is_err());
        let mut c = sample_bar();
        c.close = f64::NAN;
        assert!(validate_bar_rows(&[c]).is_err());
        let mut v = sample_bar();
        v.volume = f64::NAN;
        assert!(validate_bar_rows(&[v]).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_rejects_high_too_low_only() {
        let mut b = sample_bar();
        b.high = 9.9;
        assert!(validate_bar_rows(&[b]).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_rejects_low_too_high_only() {
        let mut b = sample_bar();
        b.low = 10.0001;
        assert!(validate_bar_rows(&[b]).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_accepts_high_at_mx_ceiling() {
        let mut b = sample_bar();
        b.open = 10.;
        b.close = 10.;
        b.high = 10. - 1e-9;
        b.low = 9.;
        assert!(validate_bar_rows(&[b]).is_ok());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_accepts_low_at_mn_floor() {
        let mut b = sample_bar();
        b.open = 10.;
        b.close = 10.;
        b.low = 10. + 1e-9;
        b.high = 12.;
        assert!(validate_bar_rows(&[b]).is_ok());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_rejects_negative_volume() {
        let b = Bar {
            date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            open: 1.,
            high: 2.,
            low: 0.5,
            close: 1.5,
            volume: -1.,
        };
        assert!(validate_bar_rows(&[b]).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_bar_rows_rejects_non_finite() {
        let b = Bar {
            date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            open: f64::INFINITY,
            high: 2.,
            low: 0.5,
            close: 1.5,
            volume: 10.,
        };
        assert!(validate_bar_rows(&[b]).is_err());
    }

    // r[impl data.validation]
    // r[verify data.validation]
    #[test]
    fn validate_order_rejects_duplicate_dates() {
        let d = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let rows = vec![
            Bar {
                date: d,
                open: 1.,
                high: 2.,
                low: 0.5,
                close: 1.5,
                volume: 10.,
            },
            Bar {
                date: d,
                open: 1.,
                high: 2.,
                low: 0.5,
                close: 1.5,
                volume: 20.,
            },
        ];
        assert!(validate_order(&rows).is_err());
    }

    // r[impl gui.chart.anchor]
    // r[verify gui.chart.anchor]
    #[test]
    fn anchor_date_caps_to_last_csv_bar() {
        let base = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let bars = vec![Bar {
            date: base,
            open: 1.,
            high: 2.,
            low: 0.5,
            close: 1.5,
            volume: 10.,
        }];
        let far_future = NaiveDate::from_ymd_opt(2099, 1, 1).unwrap();
        assert_eq!(anchor_date(&bars, far_future), Some(base));
    }
}

// r[impl test.proptest.sma150]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_sma150 {
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use super::*;
    use crate::test_inputs::Sma150Series;

    // r[impl test.proptest.sma150]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.sma150]
        #[test]
        // r[verify test.proptest.sma150]
        // r[verify test.arbitrary.proptest]
        fn sma150_matches_naive_reference(series in arb::<Sma150Series>()) {
            let closes = &series.closes;
            let out = sma_150(closes);
            for i in 149..closes.len() {
                let expected: f64 = closes[i - 149..=i].iter().sum::<f64>() / 150.0;
                let got = out[i].expect("SMA 150 should exist");
                prop_assert!((got - expected).abs() < 1e-9);
            }
        }
    }
}

// r[impl test.proptest.sma]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_sma {
    use std::fmt::Write;

    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use super::*;
    use crate::test_inputs::{CsvRoundtripSeries, Sma50Series};

    // r[impl test.proptest.sma]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.sma]
        #[test]
        // r[verify test.proptest.sma]
        // r[verify test.arbitrary.proptest]
        fn sma_matches_naive_reference(series in arb::<Sma50Series>()) {
            let closes = &series.closes;
            let out = sma_50(closes);
            for i in 49..closes.len() {
                let expected: f64 = closes[i - 49..=i].iter().sum::<f64>() / 50.0;
                let got = out[i].expect("SMA should exist");
                prop_assert!((got - expected).abs() < 1e-9);
            }
        }

        // r[impl test.proptest.sma]
        #[test]
        // r[verify test.proptest.sma]
        fn csv_roundtrip_small_series(series in arb::<CsvRoundtripSeries>()) {
            let bars = series.0.bars();
            let n = bars.len();
            let mut s = String::from("Date,Open,High,Low,Close,Volume\n");
            for b in bars {
                writeln!(
                    s,
                    "{},{},{},{},{},{}",
                    b.date.format("%Y-%m-%d"),
                    b.open,
                    b.high,
                    b.low,
                    b.close,
                    b.volume
                )
                .unwrap();
            }
            let rows = parse_csv_str(&s).unwrap();
            prop_assert_eq!(rows.len(), n);
            let tmp = tempfile::NamedTempFile::new().unwrap();
            write_csv(tmp.path(), &rows).unwrap();
            let back = std::fs::read_to_string(tmp.path()).unwrap();
            let rows2 = parse_csv_str(&back).unwrap();
            prop_assert_eq!(rows, rows2);
        }
    }
}

// r[impl test.proptest.sma]
#[cfg(test)]
mod invalid_csv_oracles {
    // r[impl test.proptest.sma]
    use super::*;

    // r[impl test.proptest.sma]
    fn corrupt_row(base: &str, row_idx: usize, new_line: &str) -> String {
        let mut lines: Vec<&str> = base.lines().collect();
        lines[row_idx] = new_line;
        let mut s = lines.join("\n");
        s.push('\n');
        s
    }

    // r[impl test.proptest.sma]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_negative_volume_second_row() {
        let base = "Date,Open,High,Low,Close,Volume\n\
2020-01-01,10,12,9,11,100\n\
2020-01-02,11,13,10,12,200\n";
        let s = corrupt_row(base, 2, "2020-01-02,11,13,10,12,-1");
        assert!(parse_csv_str(&s).is_err());
    }

    // r[impl test.proptest.sma]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_non_finite_close() {
        let base = "Date,Open,High,Low,Close,Volume\n\
2020-01-01,10,12,9,11,100\n\
2020-01-02,11,13,10,12,200\n";
        let s = corrupt_row(base, 1, "2020-01-01,10,12,9,inf,100");
        assert!(parse_csv_str(&s).is_err());
    }

    // r[impl test.proptest.sma]
    // r[verify data.validation]
    #[test]
    fn parse_rejects_unsorted_dates_oracle() {
        let s = "Date,Open,High,Low,Close,Volume\n\
2020-01-02,11,13,10,12,200\n\
2020-01-01,10,12,9,11,100\n";
        assert!(parse_csv_str(s).is_err());
    }
}
