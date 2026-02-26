use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use f1_term_core::client::{F1Client, TelemetryEvent};
use ratatui::{DefaultTerminal, Frame};
use tokio::{sync::mpsc, time::interval};

use crate::{
    action::Action, components::Component, pages::live_timing::LiveTimingPage, state::AppState,
};

const REFRESH_RATE_MILLIS: u64 = 200;

pub struct App<C: F1Client> {
    client: C,
    app_state: AppState,
    session: Option<std::sync::Arc<f1_term_core::session::Session>>,
    live_timing_page: LiveTimingPage,
}

impl<C: F1Client> App<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            app_state: AppState::default(),
            session: None,
            live_timing_page: LiveTimingPage::default(),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut render_interval = interval(Duration::from_millis(REFRESH_RATE_MILLIS));

        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        self.live_timing_page.init()?;

        while !self.app_state.exit {
            tokio::select! {
                update = self.client.next_event() => {
                    if let Some(TelemetryEvent::SessionUpdate(fs)) = update {
                        action_tx.send(Action::SessionUpdate(fs))?;
                    }
                }

                _ = render_interval.tick() => {
                    action_tx.send(Action::Tick)?;
                }

                Some(action) = action_rx.recv() => {
                    if let Some(new_action) = self.update(action.clone())? {
                        action_tx.send(new_action)?;
                    }

                    if matches!(action, Action::Render | Action::SessionUpdate(_) | Action::ToggleGapMode) {
                        terminal.draw(|frame| self.render(frame).unwrap())?;
                    }

                    if self.app_state.exit {
                        break;
                    }
                }
            }

            if event::poll(Duration::from_millis(0))?
                && let Event::Key(key) = event::read()?
            {
                action_tx.send(Action::KeyPress(key))?;
            }
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        match &action {
            Action::Quit => {
                self.app_state.exit = true;
                return Ok(None);
            }
            Action::SessionUpdate(session) => {
                self.session = Some(session.clone());
            }
            Action::KeyPress(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(Some(Action::Quit));
                }
                _ => {}
            },
            _ => {}
        }

        self.live_timing_page.update(action)
    }

    fn render(&mut self, frame: &mut Frame) -> Result<(), Box<dyn std::error::Error>> {
        self.live_timing_page.draw(frame, frame.area())?;
        Ok(())
    }
}
