use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::action::Action;
use crate::components::Component;
use crate::components::circuit_canvas::CircuitCanvas;
use crate::components::help_popup::HelpPopup;
use crate::components::message_log::MessageLog;
use crate::components::timing_table::TimingTable;
use crate::components::title_bar::TitleBar;

#[derive(Default)]
pub struct DashboardPage {
    title_bar: TitleBar,
    table: TimingTable,
    message_log: MessageLog,
    circuit_canvas: CircuitCanvas,
    help_popup: HelpPopup,
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
        if let Some(Action::Render) = self.circuit_canvas.update(action.clone())? {
            should_render = true;
        }
        if let Some(Action::Render) = self.help_popup.update(action)? {
            should_render = true;
        }

        if should_render {
            Ok(Some(Action::Render))
        } else {
            Ok(None)
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let [title, table, bottom] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(23), // number of drivers + 1
            Constraint::Fill(0),
        ])
        .areas(area);

        let [circuit_canvas, messages] =
            Layout::horizontal([Constraint::Percentage(33), Constraint::Percentage(67)])
                .areas(bottom);

        self.title_bar.draw(frame, title)?;
        self.table.draw(frame, table)?;
        self.message_log.draw(frame, messages)?;
        self.circuit_canvas.draw(frame, circuit_canvas)?;
        self.help_popup.draw(frame, area)?;

        Ok(())
    }
}
