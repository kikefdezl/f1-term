use f1_term_core::driver::Driver;
use f1_term_core::team::Team;
use ratatui::layout::Constraint;
use ratatui::style::Color;
use ratatui::widgets::{Row, Table as RatatuiTable, Widget};

pub struct TableData {
    driver_tla: String,
    driver_number: String,
    team_color: Color,
}

impl TableData {
    pub fn from_driver_team(driver: &Driver, team: &Team) -> Self {
        TableData {
            driver_tla: driver.tla.clone(),
            driver_number: driver.number.value.to_string(),
            team_color: Color::from_u32(team.color.u32),
        }
    }

    fn ref_array(&self) -> [&str; 2] {
        [&self.driver_tla, &self.driver_number]
    }
}

#[derive(Default)]
pub struct Table {
    items: Vec<TableData>,
}

impl Table {
    pub fn new(items: Vec<TableData>) -> Self {
        Table { items }
    }
}

impl Widget for Table {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rows: Vec<Row> = self
            .items
            .iter()
            .map(|data| data.ref_array().into_iter().collect())
            .collect();

        let t = RatatuiTable::new(
            rows,
            [
                Constraint::Length(4), // 3 from TLA + 1 for padding
                Constraint::Min(3),
            ],
        );
        t.render(area, buf);
    }
}
