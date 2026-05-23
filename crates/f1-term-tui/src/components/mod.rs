use ratatui::Frame;
use ratatui::layout::Rect;

use crate::action::Action;

pub mod bottom_bar;
pub mod circuit_canvas;
pub mod help_popup;
pub mod message_log;
pub mod spread_bar;
pub mod stint_table;
pub mod timing_table;
pub mod title_bar;

pub trait Component {
    fn update(&mut self, _action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        Ok(None)
    }

    fn draw(&mut self, _f: &mut Frame, _area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
