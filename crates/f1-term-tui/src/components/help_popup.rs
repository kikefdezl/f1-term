use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::action::Action;
use crate::components::Component;
use crate::constants::{
    COLOR_ABORTED, COLOR_IN_PIT, COLOR_OVERALL_FASTEST, COLOR_PERSONAL_FASTEST, COLOR_SLOWER,
    SEGMENTS,
};

#[derive(Default)]
pub struct HelpPopup {
    pub visible: bool,
}

impl Component for HelpPopup {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::KeyPress(key) = action {
            match key.code {
                KeyCode::Char('?') => {
                    self.visible = !self.visible;
                    return Ok(Some(Action::Render));
                }
                KeyCode::Esc => {
                    if self.visible {
                        self.visible = false;
                        return Ok(Some(Action::Render));
                    }
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        if !self.visible {
            return Ok(());
        }

        let shortcuts = [
            ("Q", "Quit"),
            ("?", "Toggle Help"),
            ("G", "Toggle Gap/Int"),
            ("N", "Toggle Corner Numbers"),
            ("←", "Increase Live Delay"),
            ("→", "Decrease Live Delay"),
            // TODO: These are not useful for anything yet:
            // ("↑/↓", "Select Driver (in Timing Table)"),
            // ("Esc", "Deselect / Close Help"),
        ];

        let segment_helps = [
            (COLOR_SLOWER, "Slower"),
            (COLOR_PERSONAL_FASTEST, "Personal Fastest"),
            (COLOR_OVERALL_FASTEST, "Overall Fastest"),
            (COLOR_IN_PIT, "In Pit"),
            (COLOR_ABORTED, "Aborted"),
        ];

        let mut lines = vec![];
        for (key, desc) in shortcuts {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>5}  ", key),
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("- ", Style::default().fg(Color::DarkGray)),
                Span::styled(desc.to_string(), Style::default().fg(Color::White)),
            ]));
        }
        lines.push(Line::default());
        for (color, desc) in segment_helps {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>6} ", SEGMENTS.to_string()),
                    Style::default().fg(color),
                ),
                Span::styled("- ", Style::default().fg(Color::DarkGray)),
                Span::styled(desc.to_string(), Style::default().fg(Color::White)),
            ]))
        }

        let popup_width = 45;
        let popup_height = lines.len() as u16 + 2;

        let [center_area] = Layout::horizontal([Constraint::Length(popup_width)])
            .flex(Flex::Center)
            .areas(area);

        let [popup_area] = Layout::vertical([Constraint::Length(popup_height)])
            .flex(Flex::Center)
            .areas(center_area);

        let block = Block::default()
            .title(" Help / Shortcuts ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightRed));

        let p = Paragraph::new(lines).block(block);

        frame.render_widget(Clear, popup_area);
        frame.render_widget(p, popup_area);

        Ok(())
    }
}
