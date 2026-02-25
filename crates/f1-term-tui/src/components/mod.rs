use ratatui::{Frame, layout::Rect};

use crate::action::Action;

pub mod table;

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
