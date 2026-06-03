// r[impl app.errors]

use thiserror::Error;

// r[impl app.errors]
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("validation: {0}")]
    Validation(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Twelve Data API: {0}")]
    TwelveData(String),

    #[error(
        "Schwab download is not implemented yet — complete r[cli.download.schwab] checklist in spec"
    )]
    SchwabNotImplemented,

    #[error("date parse: {0}")]
    DateParse(String),

    #[cfg(feature = "twelve-data")]
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// r[impl app.errors]
pub type Result<T> = std::result::Result<T, Error>;
