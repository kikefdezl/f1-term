use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::prelude::Rect;
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::Component;

#[derive(Default)]
pub struct BottomBar;

impl Component for BottomBar {
    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let line = Line::from(vec![
            Span::raw("<?>").fg(Color::Red).bold(),
            Span::raw(" Help  ").dim(),
        ]);
        let paragraph = Paragraph::new(line).alignment(Alignment::Right);
        f.render_widget(paragraph, area);
        Ok(())
    }
}
