use std::{sync::Arc, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use f1_term_core::{
    client::{F1Client, TelemetryEvent},
    session::Session,
};
use ratatui::{DefaultTerminal, Frame};
use tokio::time::interval;

use super::{
    state::AppState,
    table::{Table, TableData, TableDataArgs},
};

pub struct App<C: F1Client> {
    client: C,
    app_state: AppState,
    session_state: Option<Arc<Session>>,
}

impl<C: F1Client> App<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            session_state: None,
            app_state: AppState::default(),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut render_interval = interval(Duration::from_millis(333));

        while !self.app_state.exit {
            tokio::select! {
                update = self.client.next_event() => {
                    if let Some(TelemetryEvent::SessionUpdate(fs)) = update {
                        self.session_state = Some(fs);
                    }
                }

                _ = render_interval.tick() => {
                    if event::poll(Duration::from_millis(0))? &&
                        let Event::Key(key) = event::read()? {
                            self.handle_key_press(key);
                    }

                    if let Some(state) = self.session_state.as_ref() &&
                        !state.drivers.is_empty() && !state.teams.is_empty() {
                            terminal.draw(|frame| self.render(frame, state))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn handle_key_press(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.app_state.exit = true,
            KeyCode::Char('g') => self.app_state.gap_mode.toggle(),
            _ => {}
        }
    }

    pub fn render(&self, frame: &mut Frame, session: &Session) {
        let table_datas = {
            let mut tds = Vec::new();
            for participant in session.leaderboard() {
                let args = TableDataArgs {
                    driver: participant.driver,
                    team: participant.team,
                    live_timing: participant.timing,
                    stints: participant.stints,
                };
                tds.push(TableData::from(&args));
            }
            tds
        };
        let table = Table::new(table_datas, self.app_state.gap_mode);
        frame.render_widget(table, frame.area());
    }
}
