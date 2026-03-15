use std::error::Error;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::action::Action;
use crate::components::Component;
use crate::components::stint_table::StintTable;
use crate::components::title_bar::TitleBar;

#[derive(Default)]
pub struct StintsPage {
    title_bar: TitleBar,
    stint_table: StintTable,
}

impl Component for StintsPage {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn Error>> {
        let mut should_render = false;
        if let Ok(Some(Action::Render)) = self.title_bar.update(action.clone()) {
            should_render = true;
        }
        if let Ok(Some(Action::Render)) = self.stint_table.update(action) {
            should_render = true;
        }
        if should_render {
            return Ok(Some(Action::Render));
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<(), Box<dyn Error>> {
        let [title, table] =
            Layout::vertical([Constraint::Length(3), Constraint::Length(23)]).areas(area);
        self.title_bar.draw(f, title)?;
        self.stint_table.draw(f, table)
    }
}
