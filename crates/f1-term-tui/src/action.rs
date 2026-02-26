use std::sync::Arc;

use f1_term_core::session::Session;

use crate::pages::ActivePage;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    KeyPress(crossterm::event::KeyEvent),
    ToggleGapMode,
    SessionUpdate(Arc<Session>),
    Navigate(ActivePage),
    Quit,
}

impl Action {
    pub fn should_rerender(&self) -> bool {
        match self {
            Action::Tick => false,
            Action::Render => true,
            Action::Resize(_, _) => true,
            Action::KeyPress(_) => false,
            Action::ToggleGapMode => true,
            Action::SessionUpdate(_) => true,
            Action::Navigate(_) => true,
            Action::Quit => false,
        }
    }
}
