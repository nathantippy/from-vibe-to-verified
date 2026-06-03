// r[impl app.identity]
// r[impl app.overview]

use clap::Parser;

use stockviz::cli::{Cli, Command};
use stockviz::data;

#[cfg(feature = "gui")]
use eframe::egui;

// r[impl app.errors]
fn main() {
    init_tracing_and_log();
    if let Err(e) = run() {
        log::error!("fatal: {e} ({e:?})");
        tracing::error!(error = %e, error_debug = ?e, "fatal");
        std::process::exit(1);
    }
}

// r[impl app.logging]
// r[impl app.logging.tracing]
fn init_tracing_and_log() {
    let _ = flexi_logger::Logger::try_with_env_or_str("info,stockviz=debug")
        .map(|l| l.format(flexi_logger::detailed_format).start());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,stockviz=debug".into()),
        )
        .try_init();
}

// r[impl app.overview]
fn run() -> stockviz::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        // r[impl gui.core]
        Command::Graph { path } => run_graph(path)?,
        #[cfg(feature = "twelve-data")]
        Command::Download { symbol, api_key } => {
            stockviz::download::download_to_csv(&symbol, api_key.as_deref())?;
        }
        #[cfg(not(feature = "twelve-data"))]
        Command::Download { symbol } => {
            stockviz::download::download_to_csv(&symbol, None)?;
        }
    }
    Ok(())
}

// r[impl cli.graph.path]
// r[impl gui.core]
#[cfg(feature = "gui")]
fn run_graph(path: std::path::PathBuf) -> stockviz::Result<()> {
    let csv_path = data::resolve_graph_csv_path(&path)?;
    let bars = data::load_csv(&csv_path)?;
    if bars.is_empty() {
        return Err(stockviz::Error::Validation("CSV has no data rows".into()));
    }
    let label = csv_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("chart");
    let title = format!("StockViz — {label}");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "StockViz",
        options,
        Box::new(|cc| Ok(Box::new(stockviz::gui::StockvizApp::new(cc, bars)))),
    )
    .map_err(|e| stockviz::Error::Validation(format!("eframe: {e}")))?;
    Ok(())
}

// r[impl gui.core]
#[cfg(not(feature = "gui"))]
fn run_graph(_path: std::path::PathBuf) -> stockviz::Result<()> {
    Err(stockviz::Error::Validation(
        "graph subcommand requires the `gui` feature (rebuild with default features)".into(),
    ))
}
