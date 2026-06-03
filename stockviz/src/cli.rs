// r[impl cli]
// r[impl cli.commands]
use std::path::PathBuf;

use clap::{Parser, Subcommand};

// r[impl cli]
#[derive(Parser, Debug)]
#[command(name = "stockviz", version, about = "StockViz — see stock_viz_spec.md")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

// r[impl cli.commands]
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Download daily OHLCV to `<SYMBOL>.csv` in the current directory.
    // r[impl cli.download]
    Download {
        symbol: String,
        /// Twelve Data API key (overrides `TWELVE_DATA_API_KEY`). Absent on `schwab` builds.
        #[cfg(feature = "twelve-data")]
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Open a CSV chart in a maximized window.
    // r[impl app.overview]
    Graph {
        /// Symbol or path to CSV (`NOW` → `NOW.csv`; see `r[cli.graph.path]`).
        path: PathBuf,
    },
}

// r[impl cli.commands]
#[cfg(test)]
mod tests {
    use super::*;

    // r[impl cli.commands]
    // r[verify cli.commands]
    // r[verify app.overview]
    #[test]
    fn parses_graph_subcommand() {
        let cli = Cli::try_parse_from(["stockviz", "graph", "x.csv"]).unwrap();
        match cli.command {
            Command::Graph { path } => assert_eq!(path.as_os_str(), "x.csv"),
            _ => panic!("expected graph"),
        }
    }

    // r[impl cli.commands]
    #[cfg(feature = "twelve-data")]
    #[test]
    fn parses_download_with_api_key() {
        let cli = Cli::try_parse_from(["stockviz", "download", "IBM", "--api-key", "k"]).unwrap();
        match cli.command {
            Command::Download { symbol, api_key } => {
                assert_eq!(symbol, "IBM");
                assert_eq!(api_key.as_deref(), Some("k"));
            }
            _ => panic!("expected download"),
        }
    }

    // r[impl cli.graph.path]
    // r[verify cli.graph.path]
    #[test]
    fn parses_graph_symbol_without_extension() {
        let cli = Cli::try_parse_from(["stockviz", "graph", "NOW"]).unwrap();
        match cli.command {
            Command::Graph { path } => assert_eq!(path.as_os_str(), "NOW"),
            _ => panic!("expected graph"),
        }
    }

    // r[impl cli.commands]
    #[cfg(not(feature = "twelve-data"))]
    #[test]
    fn parses_download_schwab_build() {
        let cli = Cli::try_parse_from(["stockviz", "download", "IBM"]).unwrap();
        match cli.command {
            Command::Download { symbol } => assert_eq!(symbol, "IBM"),
            _ => panic!("expected download"),
        }
    }
}
