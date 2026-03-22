use ratatui::DefaultTerminal;

use crate::Result;
use crate::api::models::SessionIndex;
use crate::pages::playback::PlaybackPage;
use crate::pages::selection::SelectionPage;
use crate::pages::{ActivePage, Page};

pub enum Action {
    UpdateSession(Option<SessionIndex>),
    Activate(ActivePage),
    Quit,
}

#[derive(Default)]
pub struct App {
    active_page: ActivePage,
    selection_page: SelectionPage,
    playback_page: PlaybackPage,
}

impl App {
    pub fn new() -> App {
        App {
            active_page: ActivePage::Selection,
            selection_page: SelectionPage::new(),
            playback_page: PlaybackPage::new(),
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            let action = match self.active_page {
                ActivePage::Selection => self.selection_page.run(terminal).await?,
                ActivePage::Playback => self.playback_page.run(terminal).await?,
            };

            match action {
                Action::UpdateSession(Some(session)) => {
                    self.active_page = ActivePage::Playback;
                    self.playback_page.start(session);
                }
                Action::UpdateSession(None) | Action::Quit => break,
                Action::Activate(page) => self.active_page = page,
            }
        }

        Ok(())
    }
}
