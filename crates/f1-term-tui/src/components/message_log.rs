use f1_term_core::flag::FlagColor;
use f1_term_core::race_control_message::{MessageCategory, RaceControlMessage};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use super::Component;
use crate::action::Action;

#[derive(Default)]
pub struct MessageLog {
    messages: Vec<RaceControlMessage>,
}

impl Component for MessageLog {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::StateUpdate(ref state_lock) = action {
            let state = state_lock.read().unwrap();
            let new_len = state.race_control_messages.len();
            let old_len = self.messages.len();

            if new_len > 0 && new_len != old_len {
                self.messages.clone_from(&state.race_control_messages);
                return Ok(Some(Action::Render));
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let lines: Vec<Line> = self
            .messages
            .iter()
            .rev()
            .map(|msg| {
                let time_str = msg.timestamp.format("%H:%M:%S").to_string();
                let time_span = Span::styled(time_str, Style::default().fg(Color::DarkGray));

                let (prefix, color) = match &msg.category {
                    MessageCategory::Flag(flag) => {
                        let c = match flag.color {
                            FlagColor::Green | FlagColor::Clear => Color::Green,
                            FlagColor::Yellow | FlagColor::DoubleYellow => Color::Yellow,
                            FlagColor::Red => Color::Red,
                            FlagColor::Chequered => Color::White,
                            FlagColor::Blue => Color::Blue,
                            FlagColor::BlackAndWhite => Color::White,
                        };
                        (Span::styled("  ", Style::default().fg(c)), c)
                    }
                    MessageCategory::SafetyCar => (
                        Span::styled(" ⟨SC⟩ ", Style::default().fg(Color::Yellow).bold()),
                        Color::Yellow,
                    ),
                    MessageCategory::Other => (
                        Span::styled(" 󰋼 ", Style::default().fg(Color::Blue)),
                        Color::White,
                    ),
                };

                let msg_span = Span::styled(&msg.message, Style::default().fg(color));

                Line::from(vec![time_span, prefix, msg_span])
            })
            .collect();

        let block = Block::default()
            .title(" Race Control Messages ")
            .border_style(Style::default().fg(Color::Gray))
            .border_type(BorderType::Rounded);

        let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

        frame.render_widget(p, area);

        Ok(())
    }
}
