use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

use crate::{
    action::Action,
    components::{
        Component, circuit_canvas::CircuitCanvas, message_log::MessageLog,
        timing_table::TimingTable, title_bar::TitleBar,
    },
};

#[derive(Default)]
pub struct DashboardPage {
    title_bar: TitleBar,
    table: TimingTable,
    message_log: MessageLog,
    circuit_canvas: CircuitCanvas,
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
        if let Some(Action::Render) = self.message_log.update(action.clone())? {
            should_render = true;
        }
        if let Some(Action::Render) = self.circuit_canvas.update(action)? {
            should_render = true;
        }

        if should_render {
            Ok(Some(Action::Render))
        } else {
            Ok(None)
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let [title, rest] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        let [table, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Percentage(25)]).areas(rest);

        let [circuit_canvas, messages] = Layout::vertical([
            // height = (width / 3) tends to produce a 1:1 ratio to not distort the circuit
            Constraint::Length(right.width / 3),
            Constraint::Fill(1),
        ])
        .areas(right);

        self.title_bar.draw(frame, title)?;
        self.table.draw(frame, table)?;
        self.message_log.draw(frame, messages)?;
        self.circuit_canvas.draw(frame, circuit_canvas)?;

        Ok(())
    }
}
