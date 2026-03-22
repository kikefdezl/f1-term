pub mod playback;
pub mod selection;

use ratatui::DefaultTerminal;

use crate::Result;
use crate::app::Action;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ActivePage {
    #[default]
    Selection,
    Playback,
}

pub trait Page {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> impl std::future::Future<Output = Result<Action>> + Send;
}
