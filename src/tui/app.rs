use std::collections::HashMap;
use std::io::stdout;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table};

use super::to_color;
use super::widgets::PacketTableState;
use crate::flow::{Flow, FlowKey};
use crate::loader::{FlowLoadController, FlowLoadStatus};
use crate::tui::theme::flexoki;
use tracing::{debug, info, warn};

pub struct AppState {
    packet_table: PacketTableState,
    table_state: ratatui::widgets::TableState,
    filter: String,
    filter_mode: bool,
}

impl AppState {
    pub fn new(flows: HashMap<FlowKey, Flow>, start_timestamp: Option<f64>) -> Self {
        let mut table_state = ratatui::widgets::TableState::default();
        if !flows.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            packet_table: PacketTableState::new(flows, start_timestamp),
            table_state,
            filter: String::new(),
            filter_mode: false,
        }
    }
}

pub fn run_tui(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!(path = ?path, "Starting TUI application");
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut loader = FlowLoadController::new(path);
    let mut loading_progress = Some(0.0);
    let mut error_message: Option<String> = None;

    let mut app = AppState::new(HashMap::new(), None);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(100);

    loop {
        // Check loader
        match loader.poll() {
            FlowLoadStatus::Loading { progress } => {
                loading_progress = Some(progress);
                debug!(progress, "TUI loader progress");
            }
            FlowLoadStatus::Ready {
                flows,
                start_timestamp,
            } => {
                app = AppState::new(flows, start_timestamp);
                loading_progress = None;
                info!("TUI loader ready");
            }
            FlowLoadStatus::Error(err) => {
                error_message = Some(err);
                loading_progress = None;
                warn!("TUI loader failed");
            }
            FlowLoadStatus::Idle => {}
        }

        terminal.draw(|f| {
            if let Some(progress) = loading_progress {
                let area = f.area();
                let gauge_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(45),
                        Constraint::Length(3),
                        Constraint::Percentage(45),
                    ])
                    .split(area)[1];

                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Loading PCAP"))
                    .gauge_style(Style::default().fg(to_color(flexoki::BLUE_400)))
                    .percent((progress * 100.0) as u16);
                f.render_widget(gauge, gauge_area);
                return;
            }

            if let Some(err) = &error_message {
                let p = Paragraph::new(err.clone())
                    .block(Block::default().borders(Borders::ALL).title("Error"))
                    .style(Style::default().fg(to_color(flexoki::RED_400)));
                f.render_widget(p, f.area());
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3), // Filter
                    Constraint::Min(0),    // Table
                    Constraint::Length(3), // Footer
                ])
                .split(f.area());

            // Filter box
            let filter_title = if app.filter_mode {
                "Filter (ESC to exit)"
            } else {
                "Filter"
            };
            let filter_display = if app.filter.is_empty() && !app.filter_mode {
                "Type / to start filtering...".to_string()
            } else {
                app.filter.clone()
            };
            let filter_widget = Paragraph::new(filter_display)
                .block(Block::default().borders(Borders::ALL).title(filter_title));
            f.render_widget(filter_widget, chunks[0]);

            // Table - get filtered data
            let (rows, widths) = {
                let (r, w) = app.packet_table.get_filtered_table_data(&app.filter);
                (r, w)
            };
            let header_cells = [
                "Timestamp",
                "Src IP",
                "Src Port",
                "Dst IP",
                "Dst Port",
                "Protocol",
                "Packets",
                "Bytes",
            ]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells)
                .height(1)
                .bg(to_color(flexoki::BLUE_600));

            let table = Table::new(rows, widths)
                .header(header)
                .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ")
                .block(Block::default().borders(Borders::ALL).title("Flows"));
            f.render_stateful_widget(table, chunks[1], &mut app.table_state);

            // Footer with instructions
            let instructions = if app.filter_mode {
                Paragraph::new("Type to filter | ESC: Exit filter | Enter: Apply filter")
            } else {
                Paragraph::new("↑/↓: Navigate | Enter/Space: Expand/Collapse | /: Filter | q: Quit")
            }
            .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(instructions, chunks[2]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if loading_progress.is_some() || error_message.is_some() {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                            info!("TUI quit requested while loading/error state");
                            break;
                        }
                    } else if app.filter_mode {
                        // Handle filter input mode
                        match key.code {
                            KeyCode::Esc => {
                                app.filter_mode = false;
                                debug!("Exited filter mode");
                            }
                            KeyCode::Enter => {
                                app.filter_mode = false;
                                // Reset table selection when filter changes
                                app.table_state.select(Some(0));
                                debug!("Applied filter text");
                            }
                            KeyCode::Backspace => {
                                app.filter.pop();
                            }
                            KeyCode::Char(c) => {
                                app.filter.push(c);
                            }
                            _ => {}
                        }
                    } else {
                        // Handle normal navigation mode
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                info!("TUI quit requested");
                                break;
                            }
                            KeyCode::Char('/') => {
                                app.filter_mode = true;
                                debug!("Entered filter mode");
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.packet_table.next_flow(&mut app.table_state);
                                debug!("Moved selection down");
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.packet_table.previous_flow(&mut app.table_state);
                                debug!("Moved selection up");
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                app.packet_table.toggle_selected_flow(&app.table_state);
                                debug!("Toggled flow details");
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    info!("TUI application exited");
    Ok(())
}
