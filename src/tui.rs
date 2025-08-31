#[cfg(feature = "tui")]
mod tui_impl {
    use std::io::stdout;
    use std::time::{Duration, Instant};

    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use crossterm::{execute, terminal};
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph};
    use tui_tree_widget::{Tree, TreeItem, TreeState};

    use crate::flow::{Flow, FlowKey};

    pub struct AppState {
        flows: std::collections::HashMap<FlowKey, Flow>,
        // Items are sorted display strings paired with their FlowKey
        items: Vec<(String, FlowKey)>,
        filter: String,
        // Tree view state (selection + expansion)
        tree_state: TreeState<String>,
    }

    impl AppState {
        pub fn new(flows: std::collections::HashMap<FlowKey, Flow>) -> Self {
            let mut items: Vec<(String, FlowKey)> =
                flows.keys().map(|k| (k.to_display(), *k)).collect();
            items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            AppState {
                flows,
                items,
                filter: String::new(),
                tree_state: TreeState::default(),
            }
        }

        fn filtered_indices(&self) -> Vec<usize> {
            if self.filter.is_empty() {
                return (0..self.items.len()).collect();
            }
            let needle = self.filter.to_lowercase();
            self.items
                .iter()
                .enumerate()
                .filter(|(_, (s, _))| s.to_lowercase().contains(&needle))
                .map(|(i, _)| i)
                .collect()
        }

        fn select_first_flow(&mut self) {
            let filtered = self.filtered_indices();
            if let Some(first) = filtered.first() {
                self.tree_state.select(vec![format!("flow-{}", first)]);
            } else {
                // Clear selection if nothing is filtered
                self.tree_state.select(vec![]);
            }
        }

        fn selected_flow_pos(&self) -> Option<usize> {
            let filtered = self.filtered_indices();
            let first = self.tree_state.selected().first()?.clone();
            let idx = first.strip_prefix("flow-")?.parse::<usize>().ok()?;
            filtered.iter().position(|&v| v == idx)
        }

        fn select_flow_by_pos(&mut self, pos: usize) {
            let filtered = self.filtered_indices();
            if pos < filtered.len() {
                let id = format!("flow-{}", filtered[pos]);
                self.tree_state.select(vec![id]);
            }
        }

        fn toggle_current_flow(&mut self) {
            if let Some(first) = self.tree_state.selected().first().cloned() {
                if first.starts_with("flow-") {
                    self.tree_state.toggle(vec![first.clone()]);
                    // Keep selection anchored on the flow after toggling
                    self.tree_state.select(vec![first]);
                }
            }
        }
    }

    pub fn run_tui(
        flows: std::collections::HashMap<FlowKey, Flow>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // title
                        Constraint::Length(3), // filter
                        Constraint::Min(0),    // list / packets
                    ])
                    .split(size);

                let title = Paragraph::new(Line::from(Span::styled(
                    "Wirecrab (TUI) — q/Esc to quit, Enter to expand/collapse, arrows to navigate",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                f.render_widget(title, chunks[0]);

                let filter = Paragraph::new(format!("{}", app.filter))
                    .block(Block::default().borders(Borders::ALL).title("Filter"));
                f.render_widget(filter, chunks[1]);

                // Build tree of flows (top-level) -> packets (children)
                let filtered = app.filtered_indices();
                let mut tree_items: Vec<TreeItem<'static, String>> =
                    Vec::with_capacity(filtered.len());
                // Determine which flow is selected (first element of the selected path)
                let selected_flow_id: Option<String> = app.tree_state.selected().first().cloned();
                for &idx in &filtered {
                    let (label, key) = &app.items[idx];
                    let flow = app.flows.get(key);
                    let mut children: Vec<TreeItem<'static, String>> = Vec::new();
                    if let Some(flow) = flow {
                        for (i, pkt) in flow.packets.iter().enumerate() {
                            let len = pkt.len();
                            let mut hex: String = String::new();
                            for b in pkt.iter().take(16) {
                                use std::fmt::Write as _;
                                let _ = write!(&mut hex, "{:02X} ", b);
                            }
                            if pkt.len() > 16 {
                                hex.push_str("...");
                            }
                            let child_id = format!("pkt-{}-{}", idx, i + 1);
                            let child =
                                TreeItem::new_leaf(child_id, format!("{} bytes | {}", len, hex));
                            children.push(child);
                        }
                    }
                    let node_id = format!("flow-{}", idx);
                    // Style the flow label if selected (or any of its children is selected)
                    let text = if selected_flow_id.as_deref() == Some(node_id.as_str()) {
                        Line::from(Span::styled(
                            label.clone(),
                            Style::default().add_modifier(Modifier::REVERSED),
                        ))
                    } else {
                        Line::from(label.clone())
                    };
                    let node = TreeItem::new(node_id.clone(), text, children)
                        .expect("failed to build flow tree item");
                    tree_items.push(node);
                }
                // Ensure there's a selection when none exists
                if app.tree_state.selected().is_empty() {
                    app.select_first_flow();
                }
                let tree = Tree::new(tree_items.as_slice())
                    .expect("failed to build tree")
                    .block(Block::default().borders(Borders::ALL).title("Flows"));
                f.render_stateful_widget(tree, chunks[2], &mut app.tree_state);
            })?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Down => {
                                if let Some(pos) = app.selected_flow_pos() {
                                    let filtered_len = app.filtered_indices().len();
                                    if filtered_len > 0 {
                                        let next = (pos + 1).min(filtered_len - 1);
                                        app.select_flow_by_pos(next);
                                    }
                                } else {
                                    app.select_first_flow();
                                }
                            }
                            KeyCode::Up => {
                                if let Some(pos) = app.selected_flow_pos() {
                                    let next = pos.saturating_sub(1);
                                    app.select_flow_by_pos(next);
                                } else {
                                    app.select_first_flow();
                                }
                            }
                            KeyCode::Left => {
                                app.toggle_current_flow();
                            }
                            KeyCode::Right => {
                                app.toggle_current_flow();
                            }
                            KeyCode::Enter => {
                                app.toggle_current_flow();
                            }
                            KeyCode::Char(c) => {
                                app.filter.push(c);
                                app.select_first_flow();
                            }
                            KeyCode::Backspace => {
                                app.filter.pop();
                                app.select_first_flow();
                            }
                            _ => {}
                        }
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        disable_raw_mode()?;
        // Leave the alternate screen
        execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
        Ok(())
    }
}

#[cfg(feature = "tui")]
pub use tui_impl::run_tui;

#[cfg(not(feature = "tui"))]
pub fn run_tui(
    _flows: std::collections::HashMap<crate::flow::FlowKey, crate::flow::Flow>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("TUI feature is disabled. Rebuild with --features tui to enable the Ratatui TUI.");
    Ok(())
}
