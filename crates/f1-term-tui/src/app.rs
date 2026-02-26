use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyCode};
use f1_term_core::client::{F1Client, TelemetryEvent};
use ratatui::{DefaultTerminal, Frame};
use tokio::{sync::mpsc, time::interval};

use crate::{
    action::Action,
    components::Component,
    pages::{ActivePage, live_timing::LiveTimingPage},
};

const REFRESH_RATE_MILLIS: u64 = 200;

pub struct App<C: F1Client> {
    client: C,
    active_page: ActivePage,
    live_timing_page: LiveTimingPage,
    exit: bool,
}

impl<C: F1Client> App<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            active_page: ActivePage::default(),
            live_timing_page: LiveTimingPage::default(),
            exit: false,
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut render_interval = interval(Duration::from_millis(REFRESH_RATE_MILLIS));

        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        self.live_timing_page.init()?;

        while !self.exit {
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

                    if action.should_rerender() {
                        terminal.draw(|frame| self.render(frame).unwrap())?;
                    }

                    if self.exit {
                        break;
                    }
                }
            }

            if event::poll(Duration::from_millis(0))? {
                match event::read()? {
                    CrosstermEvent::Key(key) => action_tx.send(Action::KeyPress(key))?,
                    CrosstermEvent::Resize(w, h) => action_tx.send(Action::Resize(w, h))?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        match &action {
            Action::Quit => {
                self.exit = true;
                return Ok(None);
            }
            Action::Navigate(page) => {
                self.active_page = *page;
                return Ok(Some(Action::Render));
            }
            Action::KeyPress(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Ok(Some(Action::Quit));
                }
                _ => {}
            },
            _ => {}
        }

        match self.active_page {
            ActivePage::LiveTiming => self.live_timing_page.update(action),
        }
    }

    fn render(&mut self, frame: &mut Frame) -> Result<(), Box<dyn std::error::Error>> {
        match self.active_page {
            ActivePage::LiveTiming => self.live_timing_page.draw(frame, frame.area())?,
        }
        Ok(())
    }
}
