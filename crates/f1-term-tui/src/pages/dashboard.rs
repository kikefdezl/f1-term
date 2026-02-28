use ratatui::{Frame, layout::Rect};

use crate::{
    action::Action,
    components::{Component, table::TableComponent},
};

#[derive(Default)]
pub struct DashboardPage {
    table: TableComponent,
}

impl Component for DashboardPage {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.table.init()?;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        self.table.update(action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        self.table.draw(frame, area)
    }
}
