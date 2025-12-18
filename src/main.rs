use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use tracing::{info, warn};
#[cfg(feature = "ui")]
use wirecrab::gui;
#[cfg(feature = "tui")]
use wirecrab::tui;
use wirecrab::logging;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the pcap file to parse
    file_path: PathBuf,

    /// Launch the Graphical User Interface
    #[arg(long)]
    ui: bool,

    /// Launch the Terminal User Interface
    #[arg(long)]
    tui: bool,

    /// Path to write log output when not logging to stdout
    #[arg(long, default_value = "wirecrab.log")]
    log_file: PathBuf,

    /// Emit logs to stdout instead of the log file
    #[arg(long)]
    log_stdout: bool,

    /// Log verbosity (error, warn, info, debug, trace)
    #[arg(long, default_value = "info", value_enum)]
    log_level: LogLevel,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing::level_filters::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing::level_filters::LevelFilter::ERROR,
            LogLevel::Warn => tracing::level_filters::LevelFilter::WARN,
            LogLevel::Info => tracing::level_filters::LevelFilter::INFO,
            LogLevel::Debug => tracing::level_filters::LevelFilter::DEBUG,
            LogLevel::Trace => tracing::level_filters::LevelFilter::TRACE,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let log_level = tracing::level_filters::LevelFilter::from(args.log_level);
    let log_file = args.log_file.clone();
    let log_guard = logging::init_logging(args.log_stdout, &log_file, log_level)?;

    info!(
        ?log_file,
        log_stdout = args.log_stdout,
        log_level = ?args.log_level,
        "Logger initialized"
    );
    info!(
        file = ?args.file_path,
        ui = args.ui,
        tui = args.tui,
        "Starting Wirecrab"
    );

    if args.ui {
        #[cfg(feature = "ui")]
        {
            gui::run_ui(args.file_path).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        #[cfg(not(feature = "ui"))]
        {
            eprintln!("Error: UI feature is not enabled. Recompile with --features ui");
        }
    } else if args.tui {
        #[cfg(feature = "tui")]
        {
            tui::run_tui(args.file_path).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        #[cfg(not(feature = "tui"))]
        {
            eprintln!("Error: TUI feature is not enabled. Recompile with --features tui");
        }
    } else {
        warn!("No UI selected. Use --ui or --tui to visualize the data.");
    }

    info!("Shutting down Wirecrab");
    drop(log_guard);
    Ok(())
}
