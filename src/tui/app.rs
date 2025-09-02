use std::collections::HashMap;
use std::io::stdout;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

use super::widgets::PacketTableState;
use crate::flow::{Flow, FlowKey};

pub struct AppState {
    packet_table: PacketTableState,
    table_state: ratatui::widgets::TableState,
    filter: String,
    filter_mode: bool,
}

impl AppState {
    pub fn new(flows: HashMap<FlowKey, Flow>) -> Self {
        let mut table_state = ratatui::widgets::TableState::default();
        if !flows.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            packet_table: PacketTableState::new(flows),
            table_state,
            filter: String::new(),
            filter_mode: false,
        }
    }
}

pub fn run_tui(flows: HashMap<FlowKey, Flow>) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(flows);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(200);

    loop {
        terminal.draw(|f| {
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
                "Filter (/ to edit)"
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
            ]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells).height(1).bg(Color::Blue);

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
                    if app.filter_mode {
                        // Handle filter input mode
                        match key.code {
                            KeyCode::Esc => {
                                app.filter_mode = false;
                            }
                            KeyCode::Enter => {
                                app.filter_mode = false;
                                // Reset table selection when filter changes
                                app.table_state.select(Some(0));
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
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('/') => {
                                app.filter_mode = true;
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.packet_table.next_flow(&mut app.table_state)
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.packet_table.previous_flow(&mut app.table_state)
                            }
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                app.packet_table.toggle_selected_flow(&app.table_state)
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
    Ok(())
}
