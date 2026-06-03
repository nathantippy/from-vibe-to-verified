// r[impl cli.download.twelvedata.api]
// HTTP: `GET https://api.twelvedata.com/time_series` with query:
// `symbol`, `interval=1day`, `outputsize=5000`, `adjustment=splits`, `apikey` (split-adjusted daily per Twelve Data).
// Demo cap: Twelve Data allows up to 5000 rows per request on this endpoint (not unbounded "full" history).

use chrono::NaiveDate;
use serde::Deserialize;
use tracing::instrument;

use crate::data::{Bar, write_csv};
use crate::error::{Error, Result};
use crate::test_inputs::MAX_PIPELINE_PAYLOAD;

/// Maximum `values` rows per Twelve Data time_series response (matches `outputsize=5000`).
// r[impl cli.download.twelvedata.api]
// r[impl test.fuzz.pipeline]
pub const MAX_TIME_SERIES_ROWS: usize = 5000;

/// Max JSON body bytes for CLI download (`r[cli.download.twelvedata.api]`).
// r[impl cli.download.twelvedata.api]
pub const MAX_DOWNLOAD_JSON_BODY: usize = 1_048_576;

// r[impl cli.download.twelvedata.api]
const DEFAULT_SERIES_URL: &str = "https://api.twelvedata.com/time_series";

// r[impl cli.download.twelvedata.api]
#[derive(Debug, Deserialize)]
struct TimeSeriesResponse {
    code: Option<i32>,
    message: Option<String>,
    #[allow(dead_code)]
    status: Option<String>,
    values: Option<Vec<ValueRow>>,
}

// r[impl cli.download.twelvedata.api]
#[derive(Debug, Deserialize)]
struct ValueRow {
    datetime: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

/// Process-global lock: download tests change cwd.
// r[impl cli.download.twelvedata]
#[cfg(test)]
pub(crate) static DOWNLOAD_CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Mock series URL for `download` / `download_to_csv` in tests (no real HTTP).
// r[impl cli.download.twelvedata]
#[cfg(test)]
pub(crate) static TEST_SERIES_URL: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

// r[impl cli.download.twelvedata]
pub fn download(symbol: &str, api_key: Option<&str>) -> Result<()> {
    // r[impl cli.download.twelvedata]
    #[cfg(test)]
    if let Ok(guard) = TEST_SERIES_URL.lock()
        && let Some(ref url) = *guard
    {
        return download_with_series_url(symbol, api_key, url);
    }
    download_with_series_url(symbol, api_key, DEFAULT_SERIES_URL)
}

// r[impl cli.download.twelvedata.api]
#[instrument(skip(api_key, series_url), fields(symbol))]
pub(crate) fn download_with_series_url(
    symbol: &str,
    api_key: Option<&str>,
    series_url: &str,
) -> Result<()> {
    let key = api_key
        .map(String::from)
        .or_else(|| std::env::var("TWELVE_DATA_API_KEY").ok())
        .ok_or_else(|| Error::TwelveData("set --api-key or TWELVE_DATA_API_KEY".into()))?;

    let sym = symbol.trim().to_uppercase();
    let url = format!(
        "{series_url}?symbol={}&interval=1day&outputsize=5000&adjustment=splits&apikey={}",
        urlencoding_encode(&sym),
        urlencoding_encode(&key)
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| Error::Http(e.to_string()))?;

    let resp = client
        .get(&url)
        .send()
        .map_err(|e| Error::Http(e.to_string()))?;

    let status = resp.status();
    let body = resp.text().map_err(|e| Error::Http(e.to_string()))?;
    if !status.is_success() {
        return Err(Error::TwelveData(format!("HTTP {status}: {body}")));
    }

    let bars = parse_time_series_json_for_download(body.as_bytes())?;

    let out = format!("{sym}.csv");
    let path = std::path::Path::new(&out);
    write_csv(path, &bars)?;
    log::info!("wrote {} rows to {}", bars.len(), path.display());
    Ok(())
}

// r[impl cli.download.twelvedata.api]
// r[impl test.fuzz.pipeline]
/// Parse Twelve Data `time_series` JSON for pipeline/fuzz (`MAX_PIPELINE_PAYLOAD`).
// r[impl test.fuzz.pipeline]
pub fn parse_time_series_json(body: &[u8]) -> Result<Vec<Bar>> {
    parse_time_series_json_with_limit(body, MAX_PIPELINE_PAYLOAD, "pipeline")
}

// r[impl cli.download.twelvedata.api]
/// Parse Twelve Data `time_series` JSON for CLI download (`MAX_DOWNLOAD_JSON_BODY`).
pub fn parse_time_series_json_for_download(body: &[u8]) -> Result<Vec<Bar>> {
    parse_time_series_json_with_limit(body, MAX_DOWNLOAD_JSON_BODY, "download")
}

// r[impl cli.download.twelvedata.api]
// r[impl test.fuzz.pipeline]
fn parse_time_series_json_with_limit(
    body: &[u8],
    max_bytes: usize,
    path: &'static str,
) -> Result<Vec<Bar>> {
    if body.len() > max_bytes {
        return Err(Error::TwelveData(format!(
            "JSON body exceeds {max_bytes} bytes ({path})"
        )));
    }
    let parsed: TimeSeriesResponse = serde_json::from_slice(body)?;
    if let Some(c) = parsed.code
        && c != 200
    {
        return Err(Error::TwelveData(
            parsed.message.unwrap_or_else(|| format!("API code {c}")),
        ));
    }
    let rows = parsed
        .values
        .ok_or_else(|| Error::TwelveData("missing `values` array".into()))?;
    if rows.len() > MAX_TIME_SERIES_ROWS {
        return Err(Error::TwelveData(format!(
            "`values` length {} exceeds {MAX_TIME_SERIES_ROWS}",
            rows.len()
        )));
    }

    let mut bars: Vec<Bar> = rows
        .into_iter()
        .filter_map(|v| value_row_to_bar(v).ok())
        .collect();

    bars.sort_by_key(|b| b.date);
    crate::data::validate_order(&bars)?;
    crate::data::validate_bar_rows(&bars)?;
    Ok(bars)
}

// r[impl cli.download.twelvedata.api]
fn urlencoding_encode(s: &str) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => write!(out, "%{b:02X}").unwrap(),
        }
    }
    out
}

// r[impl cli.download.twelvedata.api]
fn value_row_to_bar(v: ValueRow) -> Result<Bar> {
    let date_part = v.datetime.split(' ').next().unwrap_or(&v.datetime);
    let date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
        .map_err(|e| Error::DateParse(format!("{}: {e}", v.datetime)))?;
    let open: f64 = v
        .open
        .parse()
        .map_err(|_| Error::Validation("open".into()))?;
    let high: f64 = v
        .high
        .parse()
        .map_err(|_| Error::Validation("high".into()))?;
    let low: f64 = v.low.parse().map_err(|_| Error::Validation("low".into()))?;
    let close: f64 = v
        .close
        .parse()
        .map_err(|_| Error::Validation("close".into()))?;
    let volume: f64 = v
        .volume
        .parse()
        .map_err(|_| Error::Validation("volume".into()))?;
    Ok(Bar {
        date,
        open,
        high,
        low,
        close,
        volume,
    })
}

// r[impl test.proptest.download.json]
// r[impl test.arbitrary.proptest]
#[cfg(test)]
mod proptest_download_json {
    use proptest::prelude::*;
    use proptest_arbitrary_interop::arb;

    use crate::test_inputs::{TimeSeriesJsonBody, MAX_PIPELINE_PAYLOAD};

    use super::*;

    // r[impl test.proptest.download.json]
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        // r[impl test.proptest.download.json]
        // r[verify test.proptest.download.json]
        // r[verify test.arbitrary.proptest]
        #[test]
        fn download_json_dual_limits(body in arb::<TimeSeriesJsonBody>()) {
            let len = body.0.len();
            let pipeline = parse_time_series_json(&body.0);
            let download = parse_time_series_json_for_download(&body.0);

            if len > MAX_DOWNLOAD_JSON_BODY {
                prop_assert!(download.is_err());
                prop_assert!(pipeline.is_err());
            } else if len > MAX_PIPELINE_PAYLOAD {
                prop_assert!(pipeline.is_err());
                let bars = download.expect("download accepts under download cap");
                prop_assert!(!bars.is_empty());
            } else {
                let p = pipeline.expect("pipeline accepts under pipeline cap");
                let d = download.expect("download accepts under pipeline cap");
                prop_assert_eq!(p.len(), d.len());
            }
        }
    }
}

// r[impl cli.download.twelvedata]
#[cfg(test)]
mod tests {
    use mockito::Matcher;
    use std::fs;
    use std::path::PathBuf;

    use crate::test_inputs::{encode_time_series_json, valid_bars, MAX_PIPELINE_PAYLOAD};

    use super::DOWNLOAD_CWD_LOCK;
    use super::*;

    // r[impl cli.download.twelvedata]
    struct RestoreCwd {
        prev: PathBuf,
    }

    // r[impl cli.download.twelvedata]
    impl Drop for RestoreCwd {
        // r[impl cli.download.twelvedata]
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.prev);
        }
    }

    // r[impl cli.download.twelvedata.api]
    // r[verify cli.download.twelvedata.api]
    #[test]
    fn parses_sample_json_body() {
        let body = r#"{"values":[
            {"datetime":"2020-01-02","open":"10","high":"11","low":"9","close":"10.5","volume":"100"},
            {"datetime":"2020-01-01","open":"9","high":"10","low":"8","close":"10","volume":"50"}
        ]}"#;
        let bars = parse_time_series_json(body.as_bytes()).unwrap();
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].date.to_string(), "2020-01-01");
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn parse_rejects_oversized_json_body() {
        let body = vec![b' '; MAX_PIPELINE_PAYLOAD + 1];
        assert!(parse_time_series_json(&body).is_err());
    }

    // r[impl test.fuzz.pipeline]
    // r[verify test.fuzz.pipeline]
    #[test]
    fn parse_rejects_too_many_values() {
        let row = r#"{"datetime":"2020-01-01","open":"1","high":"1","low":"1","close":"1","volume":"1"}"#;
        let mut body = String::from(r#"{"values":["#);
        for i in 0..=MAX_TIME_SERIES_ROWS {
            if i > 0 {
                body.push(',');
            }
            body.push_str(row);
        }
        body.push_str("]}");
        assert!(parse_time_series_json(body.as_bytes()).is_err());
    }

    // r[impl cli.download.twelvedata.api]
    // r[verify cli.download.twelvedata.api]
    #[test]
    fn parse_download_accepts_json_over_pipeline_cap() {
        let body = encode_time_series_json(&valid_bars(800));
        assert!(body.len() > MAX_PIPELINE_PAYLOAD);
        assert!(parse_time_series_json(&body).is_err());
        let bars = parse_time_series_json_for_download(&body).unwrap();
        assert_eq!(bars.len(), 800);
    }

    // r[impl cli.download.twelvedata.api]
    // r[verify cli.download.twelvedata.api]
    #[test]
    fn parse_download_rejects_over_download_cap() {
        let body = vec![b' '; MAX_DOWNLOAD_JSON_BODY + 1];
        let err = parse_time_series_json_for_download(&body).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("download"));
    }

    // r[impl cli.download.twelvedata]
    // r[verify cli.download.twelvedata]
    // r[verify cli.download]
    #[test]
    fn mock_http_uses_outputsize_5000() {
        let _lock = DOWNLOAD_CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let _restore = RestoreCwd { prev };

        let body = r#"{"values":[
            {"datetime":"2020-01-01","open":"9","high":"10","low":"8","close":"10","volume":"50"}
        ]}"#;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/time_series")
            .match_query(Matcher::Regex("outputsize=5000".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let series_url = format!("{}/time_series", server.url());
        download_with_series_url("ibm", Some("fake-key"), &series_url).unwrap();
        mock.assert();
    }

    // r[impl cli.download.twelvedata]
    #[test]
    fn mock_http_writes_sorted_csv() {
        let _lock = DOWNLOAD_CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let _restore = RestoreCwd { prev };

        let body = r#"{"values":[
            {"datetime":"2020-01-02","open":"10","high":"11","low":"9","close":"10.5","volume":"100"},
            {"datetime":"2020-01-01","open":"9","high":"10","low":"8","close":"10","volume":"50"}
        ]}"#;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/time_series")
            .match_query(Matcher::Regex("outputsize=5000".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let series_url = format!("{}/time_series", server.url());
        download_with_series_url("ZZMCK", Some("fake-key"), &series_url).unwrap();
        mock.assert();

        let p = dir.path().join("ZZMCK.csv");
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains("2020-01-01"));
        assert!(text.lines().nth(1).unwrap().starts_with("2020-01-01"));
        assert!(text.lines().count() >= 2);
    }

    // r[impl cli.download.twelvedata.api]
    #[test]
    fn urlencoding_encode_spaces_and_symbols() {
        assert_eq!(urlencoding_encode("IBM US"), "IBM%20US");
        assert_eq!(urlencoding_encode("k&ey"), "k%26ey");
        assert_eq!(urlencoding_encode("abc"), "abc");
    }

    // r[impl cli.download.twelvedata]
    // r[verify cli.download.twelvedata]
    #[test]
    fn mock_http_rejects_non_success_status() {
        let _lock = DOWNLOAD_CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let _restore = RestoreCwd { prev };

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/time_series")
            .match_query(Matcher::Regex("symbol=IBM".to_string()))
            .with_status(500)
            .create();

        let series_url = format!("{}/time_series", server.url());
        let err = download_with_series_url("ibm", Some("fake-key"), &series_url).unwrap_err();
        mock.assert();
        assert!(matches!(err, Error::TwelveData(_)));
    }

    // r[impl cli.download.twelvedata.api]
    // r[verify cli.download.twelvedata.api]
    #[test]
    fn mock_http_rejects_api_error_code() {
        let _lock = DOWNLOAD_CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let _restore = RestoreCwd { prev };

        let body = r#"{"code":401,"message":"invalid key","values":[]}"#;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/time_series")
            .match_query(Matcher::Regex("symbol=IBM".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let series_url = format!("{}/time_series", server.url());
        let err = download_with_series_url("ibm", Some("fake-key"), &series_url).unwrap_err();
        mock.assert();
        let msg = format!("{err}");
        assert!(msg.contains("401") || msg.contains("invalid key"));
    }

    // r[impl cli.download.twelvedata]
    // r[verify cli.download]
    #[test]
    fn mock_http_url_contains_encoded_symbol() {
        let _lock = DOWNLOAD_CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let _restore = RestoreCwd { prev };

        let body = r#"{"values":[
            {"datetime":"2020-01-01","open":"9","high":"10","low":"8","close":"10","volume":"50"}
        ]}"#;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/time_series")
            .match_query(Matcher::Regex("symbol=ZZMCK".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        let series_url = format!("{}/time_series", server.url());
        download_with_series_url("zzmck", Some("fake-key"), &series_url).unwrap();
        mock.assert();
    }

    // r[impl cli.download.twelvedata]
    // r[verify cli.download.twelvedata]
    #[test]
    fn download_public_api_writes_csv() {
        let _lock = DOWNLOAD_CWD_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        let _restore = RestoreCwd { prev };

        let body = r#"{"values":[
            {"datetime":"2020-01-01","open":"9","high":"10","low":"8","close":"10","volume":"50"}
        ]}"#;

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/time_series")
            .match_query(Matcher::Regex("symbol=IBM".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();

        *TEST_SERIES_URL.lock().unwrap() = Some(format!("{}/time_series", server.url()));
        download("IBM", Some("fake-key")).unwrap();
        mock.assert();
        *TEST_SERIES_URL.lock().unwrap() = None;

        let p = dir.path().join("IBM.csv");
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.lines().count() >= 2);
    }
}
