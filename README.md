# Wirecrab

Wirecrab is a packet-capture inspector with both graphical (GPUI) and terminal (Ratatui) frontends. It parses `.pcap`/`.pcapng` files into flow summaries and packet details for interactive exploration.

## Requirements

- Rust 1.81+ (stable toolchain recommended)
- PCAP/PCAPNG capture files to inspect (sample data lives in `testdata/`)

Clone the repository and change into the workspace before running any commands:

```pwsh
PS> git clone <repo-url>
PS> cd wirecrab
```

## Building & Running the GPUI app

The graphical client lives behind the `ui` feature flag. Build or run it with:

```pwsh
PS> cargo run --features "ui" -- --ui --file-path .\testdata\win_pcap.pcapng
```

Key notes:

- `--ui` tells Wirecrab to launch the GPUI application.
- Provide the capture path as the first positional argument (or with `--file-path`).
- `cargo build --features "ui"` produces the binary without running it.

## Building & Running the TUI app

The Ratatui-based terminal interface is behind the `tui` feature flag:

```pwsh
PS> cargo run --features "tui" -- --tui .\testdata\win_pcap.pcapng
```

Tips:

- Navigation uses familiar `↑/↓`, `j/k`, `/` to filter, and `q` to quit.
- As with the GUI, `cargo build --features "tui"` is available when you only need a binary.

## Logging configuration

Wirecrab uses the [`tracing`](https://docs.rs/tracing) ecosystem for structured logging. Runtime flags control the destination and verbosity:

- `--log-level <level>`: One of `error`, `warn`, `info`, `debug`, `trace` (default: `info`).
- `--log-file <path>`: File target when not logging to stdout (default: `wirecrab.log`).
- `--log-stdout`: Redirect all logs to stdout instead of a file.

Example launching the GUI with verbose trace logs streamed to the console:

```pwsh
PS> cargo run --features "ui" -- --ui .\testdata\win_pcap.pcapng --log-level trace --log-stdout
```

Or capture debug logs to a custom file while running the TUI:

```pwsh
PS> cargo run --features "tui" -- --tui .\testdata\win_pcap.pcapng --log-level debug --log-file logs\wirecrab-debug.log
```

Log files are appended to, so clear or rotate them as needed.

## Troubleshooting

- **Missing feature errors**: Ensure you pass `--features "ui"` or `--features "tui"` to `cargo run/build` based on the frontend you want.
- **Permission errors writing logs**: Specify a writable path via `--log-file` or use `--log-stdout`.
- **Slow parsing**: Large captures may take time; progress is surfaced through the loader status bar or TUI progress gauge.

Feel free to open issues or PRs for bugs and enhancements. Happy packet sleuthing!
