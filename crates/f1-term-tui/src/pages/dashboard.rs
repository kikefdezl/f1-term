use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

use crate::{
    action::Action,
    components::{Component, message_log::MessageLog, table::TimingTable, title_bar::TitleBar},
};

#[derive(Default)]
pub struct DashboardPage {
    title_bar: TitleBar,
    table: TimingTable,
    message_log: MessageLog,
}

impl Component for DashboardPage {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.table.init()?;
        self.message_log.init()?;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        let mut should_render = false;
        if let Some(Action::Render) = self.title_bar.update(action.clone())? {
            should_render = true;
        }
        if let Some(Action::Render) = self.table.update(action.clone())? {
            should_render = true;
        }
        if let Some(Action::Render) = self.message_log.update(action)? {
            should_render = true;
        }

        if should_render {
            Ok(Some(Action::Render))
        } else {
            Ok(None)
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let top_bottom = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);
        let left_right =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(45)]).split(top_bottom[1]);

        self.title_bar.draw(frame, top_bottom[0])?;
        self.table.draw(frame, left_right[0])?;
        self.message_log.draw(frame, left_right[1])?;

        Ok(())
    }
}
