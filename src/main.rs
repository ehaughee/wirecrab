mod flow;
mod gui;
mod parser;
mod tui;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path> [--ui] [--tui]", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    println!("Parsing file: {}", file_path);
    let flows = parser::parse(file_path);

    if args.iter().any(|a| a == "--ui") {
        // Optionally launch GUI when built with `--features ui` and user passes --ui
        if let Err(e) = gui::run_ui(flows) {
            eprintln!("Failed to run UI: {e:?}");
        }
    } else if args.iter().any(|a| a == "--tui") {
        // Optionally launch TUI when built with `--features tui` and user passes --tui
        if let Err(e) = tui::run_tui(flows) {
            eprintln!("Failed to run TUI: {e:?}");
        }
    }
}
