use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod flow;
mod gui;
mod layers;
mod loader;
mod parser;
mod tui;

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Parsing file: {:?}", args.file_path);

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
        println!("No UI selected. Use --ui or --tui to visualize the data.");
    }

    Ok(())
}
