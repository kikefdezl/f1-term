use crossterm::event::{self, Event, KeyCode};
use log::error;
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::Result;
use crate::api::client::F1ApiClient;
use crate::api::models::SessionIndex;
use crate::app::Action;
use crate::pages::Page;

#[derive(Default)]
pub struct SelectionPage {
    api: F1ApiClient,
    sessions: Vec<SessionIndex>,
}

impl Page for SelectionPage {
    async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<Action> {
        if self.sessions.is_empty() {
            self.fetch_sessions().await;
        }
        let session = match self.run_selection_tui(terminal, &self.sessions) {
            Ok(Some(s)) => s,
            Ok(None) => {
                return Ok(Action::Quit);
            }
            Err(e) => {
                error!("UI Error: {}", e);
                return Err(e);
            }
        };

        Ok(Action::UpdateSession(Some(session)))
    }
}

impl SelectionPage {
    pub fn new() -> SelectionPage {
        SelectionPage {
            api: F1ApiClient::new(),
            sessions: Vec::new(),
        }
    }

    async fn fetch_sessions(&mut self) {
        let mut all_sessions = Vec::new();

        for year in [2024, 2025, 2026] {
            match self.api.get_index(year).await {
                Ok(index) => {
                    for meeting in &index.meetings {
                        for session in &meeting.sessions {
                            all_sessions.push(session.clone());
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to fetch index for year {}: {}", year, e);
                    println!("Failed to fetch index for year {}: {}", year, e);
                }
            }
        }

        all_sessions.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        self.sessions = all_sessions;
    }

    fn run_selection_tui(
        &self,
        terminal: &mut DefaultTerminal,
        sessions: &[SessionIndex],
    ) -> Result<Option<SessionIndex>> {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(f.area());

                let header = Paragraph::new("Select a Session to Replay")
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(Block::default().borders(Borders::ALL).title("F1 Replay"));
                f.render_widget(header, chunks[0]);

                let items: Vec<ListItem> = sessions
                    .iter()
                    .map(|s| ListItem::new(format!("{} - {} ({})", s.start_date, s.name, s.path)))
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Sessions"))
                    .highlight_style(
                        Style::default()
                            .bg(Color::LightGreen)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");

                f.render_stateful_widget(list, chunks[1], &mut list_state);
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i >= sessions.len().saturating_sub(1) {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    sessions.len().saturating_sub(1)
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        if let Some(i) = list_state.selected() {
                            return Ok(Some(sessions[i].clone()));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
