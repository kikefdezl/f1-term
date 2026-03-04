use std::sync::{Arc, RwLock};

use f1_term_core::telemetry_state::TelemetryState;

use crate::pages::ActivePage;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    KeyPress(crossterm::event::KeyEvent),
    StateUpdate(Arc<RwLock<TelemetryState>>),
    Navigate(ActivePage),
    Quit,
}

impl Action {
    pub fn should_trigger_render(&self) -> bool {
        match self {
            Action::Tick => false,
            Action::Render => true,
            Action::Resize(_, _) => true,
            Action::KeyPress(_) => false,
            Action::StateUpdate(_) => true,
            Action::Navigate(_) => true,
            Action::Quit => false,
        }
    }
}
