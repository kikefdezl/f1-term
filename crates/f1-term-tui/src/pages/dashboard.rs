use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
};

use crate::{
    action::Action,
    components::{Component, table::TableComponent},
};

#[derive(Default)]
pub struct DashboardPage {
    table: TableComponent,
    session_title: Option<String>,
}

impl Component for DashboardPage {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.table.init()?;
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::SessionUpdate(ref session) = action {
            let title = format!(
                " {} - {} | {} ({}) ",
                session.info.meeting.official_name,
                session.info.name,
                session.info.meeting.circuit.short_name,
                session.info.meeting.country.name
            );
            self.session_title = Some(title);
        }
        self.table.update(action)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default());

        if let Some(title) = &self.session_title {
            block = block.title(ratatui::text::Span::styled(
                title.as_str(),
                Style::default(),
            ));
        }

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        self.table.draw(frame, inner_area)
    }
}
