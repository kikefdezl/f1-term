use std::sync::Arc;

use f1_term_core::session::Session;

use crate::pages::ActivePage;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Render,
    KeyPress(crossterm::event::KeyEvent),
    ToggleGapMode,
    SessionUpdate(Arc<Session>),
    Navigate(ActivePage),
    Quit,
}
