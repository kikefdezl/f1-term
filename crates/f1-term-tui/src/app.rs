use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode};
use f1_term_core::telemetry_engine::TelemetryEngineCommand;
use f1_term_core::telemetry_state::TelemetryState;
use futures::StreamExt;
use ratatui::{DefaultTerminal, Frame};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::interval;

use crate::action::Action;
use crate::components::Component;
use crate::pages::ActivePage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::stints::StintsPage;

const REFRESH_RATE_MILLIS: u64 = 100;

pub struct App {
    state: Arc<RwLock<TelemetryState>>,
    engine_tx: UnboundedSender<TelemetryEngineCommand>,
    active_page: ActivePage,
    live_timing_page: DashboardPage,
    stint_page: StintsPage,
    exit: bool,
}

impl App {
    pub fn new(
        state: Arc<RwLock<TelemetryState>>,
        engine_tx: UnboundedSender<TelemetryEngineCommand>,
    ) -> Self {
        Self {
            state,
            engine_tx,
            active_page: ActivePage::default(),
            live_timing_page: DashboardPage::default(),
            stint_page: StintsPage::default(),
            exit: false,
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut render_interval = interval(Duration::from_millis(REFRESH_RATE_MILLIS));

        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut last_update_version = u64::MAX;

        let mut cs_event_stream = EventStream::new();

        while !self.exit {
            tokio::select! {
                _ = render_interval.tick() => {
                    action_tx.send(Action::Tick)?;

                    let state_changed = {
                        let lock = self.state.read().unwrap();
                        if lock.update_version != last_update_version {
                            last_update_version = lock.update_version;
                            true
                        } else {
                            false
                        }
                    };

                    if state_changed {
                        action_tx.send(Action::StateUpdate(Arc::clone(&self.state)))?;
                    }
                }

                Some(Ok(event)) = cs_event_stream.next() => {
                    match event {
                        CrosstermEvent::Key(key) => action_tx.send(Action::KeyPress(key))?,
                        CrosstermEvent::Resize(_, _) => action_tx.send(Action::Resize)?,
                        _ => {}
                    }
                }

                Some(action) = action_rx.recv() => {
                    if let Some(new_action) = self.update(action.clone())? {
                        action_tx.send(new_action)?;
                    }

                    if action.should_trigger_render() {
                        terminal.draw(|frame| self.render(frame).unwrap())?;
                    }

                    if self.exit {
                        break;
                    }
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
                // Force data updates when changing page since they don't load the
                // data until they are active.
                return Ok(Some(Action::StateUpdate(Arc::clone(&self.state))));
            }
            Action::KeyPress(key) => match key.code {
                KeyCode::Char('q') => return Ok(Some(Action::Quit)),
                KeyCode::F(1) => return Ok(Some(Action::Navigate(ActivePage::LiveTiming))),
                KeyCode::F(2) => return Ok(Some(Action::Navigate(ActivePage::Stints))),
                KeyCode::Left => self.engine_tx.send(TelemetryEngineCommand::IncreaseDelay(
                    Duration::from_secs(1),
                ))?,
                KeyCode::Right => self.engine_tx.send(TelemetryEngineCommand::DecreaseDelay(
                    Duration::from_secs(1),
                ))?,
                _ => {}
            },
            _ => {}
        }

        match self.active_page {
            ActivePage::LiveTiming => self.live_timing_page.update(action),
            ActivePage::Stints => self.stint_page.update(action),
        }
    }

    fn render(&mut self, frame: &mut Frame) -> Result<(), Box<dyn std::error::Error>> {
        match self.active_page {
            ActivePage::LiveTiming => self.live_timing_page.draw(frame, frame.area())?,
            ActivePage::Stints => self.stint_page.draw(frame, frame.area())?,
        }
        Ok(())
    }
}
