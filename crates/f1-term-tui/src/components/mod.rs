use ratatui::{Frame, layout::Rect};

use crate::action::Action;

pub mod circuit_canvas;
pub mod message_log;
pub mod timing_table;
pub mod title_bar;

pub trait Component {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn update(&mut self, _action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        Ok(None)
    }

    fn draw(&mut self, _f: &mut Frame, _area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
