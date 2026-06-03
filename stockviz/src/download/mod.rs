//! Market data download (compile-time provider).

// r[impl build.provider]

use crate::error::Result;

// r[impl cli.download.twelvedata]
#[cfg(feature = "twelve-data")]
pub mod twelve_data;

// r[impl cli.download]
/// Download daily history for `symbol` into `<SYMBOL>.csv` in the current working directory.
pub fn download_to_csv(symbol: &str, api_key: Option<&str>) -> Result<()> {
    #[cfg(feature = "twelve-data")]
    {
        twelve_data::download(symbol, api_key)
    }
    #[cfg(not(feature = "twelve-data"))]
    {
        // r[impl cli.download.schwab]
        let _ = (symbol, api_key);
        Err(crate::error::Error::SchwabNotImplemented)
    }
}

// r[impl cli.download.twelvedata]
// r[impl cli.download]
#[cfg(all(test, feature = "twelve-data"))]
mod twelve_data_tests {
    use mockito::Matcher;
    use std::fs;
    use std::path::PathBuf;

    use super::download_to_csv;
    use crate::download::twelve_data::{DOWNLOAD_CWD_LOCK, TEST_SERIES_URL};

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

    // r[impl cli.download]
    // r[verify cli.download]
    #[test]
    fn restore_cwd_after_scope() {
        let prev = std::env::current_dir().unwrap();
        let dir = tempfile::tempdir().unwrap();
        {
            std::env::set_current_dir(dir.path()).unwrap();
            let _restore = RestoreCwd { prev: prev.clone() };
            assert_eq!(std::env::current_dir().unwrap(), dir.path());
        }
        assert_eq!(std::env::current_dir().unwrap(), prev);
    }

    // r[impl cli.download]
    // r[verify cli.download]
    // r[verify cli.download.twelvedata]
    #[test]
    fn download_to_csv_delegates_to_twelve_data() {
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

        *TEST_SERIES_URL.lock().unwrap() = Some(format!("{}/time_series", server.url()));

        download_to_csv("zzmck", Some("fake-key")).unwrap();
        mock.assert();

        let p = dir.path().join("ZZMCK.csv");
        assert!(p.is_file());
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains("2020-01-01"));

        *TEST_SERIES_URL.lock().unwrap() = None;
    }
}

// r[impl cli.download.schwab]
#[cfg(all(test, not(feature = "twelve-data")))]
mod schwab_tests {
    use super::download_to_csv;
    use crate::error::Error;

    // r[verify cli.download]
    // r[verify cli.download.schwab]
    // r[impl cli.download.schwab]
    #[test]
    fn download_returns_not_implemented() {
        let e = download_to_csv("AAPL", None).unwrap_err();
        assert!(matches!(e, Error::SchwabNotImplemented));
    }
}
