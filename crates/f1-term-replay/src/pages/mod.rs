pub mod playback;
pub mod selection;

use async_trait::async_trait;
use ratatui::DefaultTerminal;

use crate::Result;
use crate::app::Action;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ActivePage {
    #[default]
    Selection,
    Playback,
}

#[async_trait]
pub trait Page {
    async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<Action>;
}
