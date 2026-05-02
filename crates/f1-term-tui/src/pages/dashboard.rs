use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::action::Action;
use crate::components::Component;
use crate::components::circuit_canvas::CircuitCanvas;
use crate::components::help_popup::HelpPopup;
use crate::components::message_log::MessageLog;
use crate::components::spread_bar::SpreadBar;
use crate::components::timing_table::TimingTable;
use crate::components::title_bar::TitleBar;

#[derive(Default)]
pub struct DashboardPage {
    title_bar: TitleBar,
    table: TimingTable,
    message_log: MessageLog,
    circuit_canvas: CircuitCanvas,
    spread_bar: SpreadBar,
    help_popup: HelpPopup,
}

impl Component for DashboardPage {
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
        if let Some(Action::Render) = self.spread_bar.update(action.clone())? {
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
        let [title, middle, bottom] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(24), // number of drivers + 1 for the header
            Constraint::Fill(0),
        ])
        .areas(area);

        let [table, messages] =
            Layout::horizontal([Constraint::Length(115), Constraint::Fill(1)]).areas(middle);

        let [circuit, rest] =
            Layout::horizontal([Constraint::Percentage(33), Constraint::Fill(0)]).areas(bottom);

        self.title_bar.draw(frame, title)?;
        self.table.draw(frame, table)?;
        self.message_log.draw(frame, messages)?;
        self.circuit_canvas.draw(frame, circuit)?;
        self.spread_bar.draw(frame, rest)?;
        self.help_popup.draw(frame, area)?;

        Ok(())
    }
}
