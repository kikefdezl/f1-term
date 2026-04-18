use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::Component;
use crate::action::Action;

#[derive(Default)]
pub struct SpreadBar {
    drivers: Vec<DriverData>,
}

struct DriverData {
    tla: String,
    team_color: Color,
    diff_to_fastest: Option<String>,
}

impl Component for SpreadBar {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::StateUpdate(ref state_lock) = action {
            let state = state_lock.read().unwrap();
            self.drivers = state
                .drivers
                .values()
                .map(|driver| {
                    let team = state
                        .teams
                        .get(&driver.team_name)
                        .expect("Should have the team");
                    let diff_to_fastest = state
                        .timing_data
                        .get(&driver.number)
                        .expect("Should have the driver")
                        .time_diffs
                        .to_fastest
                        .clone();
                    DriverData {
                        tla: driver.tla.clone(),
                        team_color: Color::from_u32(team.color.u32),
                        diff_to_fastest,
                    }
                })
                .collect();
            return Ok(Some(Action::Render));
        }
        Ok(None)
    }

    // TODO: Render the actual Race Spread widget
    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let paragraph = Paragraph::new("Hi");
        f.render_widget(paragraph, area);
        Ok(())
    }
}
