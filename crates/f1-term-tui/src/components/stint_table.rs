use std::error::Error;

use f1_term_core::driver::DriverNumber;
use f1_term_core::stint::Stints;
use f1_term_core::telemetry_state::TelemetryState;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Cell, Row, Table};

use crate::action::Action;
use crate::components::Component;

#[derive(Default)]
pub struct StintTableData {
    driver_tla: String,
    driver_number: DriverNumber,
    team_color: Color,
    stints: Stints,
}

impl StintTableData {
    fn position_cell(&self, pos: usize) -> Cell<'_> {
        Cell::new(format!("{}", pos))
    }

    fn tla_cell(&self) -> Cell<'_> {
        Cell::new(Span::styled(
            self.driver_tla.as_str(),
            Style::default().bold().fg(self.team_color),
        ))
    }

    fn driver_number_cell(&self) -> Cell<'_> {
        Cell::new(format!("{}", self.driver_number.value))
    }

    // TODO: properly
    fn stint_cell(&self) -> Cell<'_> {
        Cell::new(format!("{}", self.stints.len()))
    }
}

#[derive(Default)]
pub struct StintTable {
    data: Vec<StintTableData>,
}

impl Component for StintTable {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn Error>> {
        if let Action::StateUpdate(ref state_lock) = action {
            let state = state_lock.read().unwrap();
            self.update_data(&state);
            return Ok(Some(Action::Render));
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<(), Box<dyn Error>> {
        let rows = self.rows();
        let t = Table::new(
            rows,
            [
                Constraint::Length(3), // #
                Constraint::Length(4), // driver
                Constraint::Length(3), // num
                Constraint::Length(3), // stint
            ],
        );
        f.render_widget(t, area);
        Ok(())
    }
}

impl StintTable {
    fn update_data(&mut self, state: &TelemetryState) {
        let participants = state.participants();
        self.data = participants
            .iter()
            .map(|participant| StintTableData {
                driver_tla: participant.driver.tla.clone(),
                driver_number: participant.driver.number,
                team_color: Color::from_u32(participant.team.color.u32),
                stints: participant.stints.unwrap_or(&Vec::new()).to_vec(),
            })
            .collect();
    }

    fn rows(&self) -> Vec<Row<'_>> {
        self.data
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let pos = i + 1;
                Row::new([
                    data.position_cell(pos),
                    data.tla_cell(),
                    data.driver_number_cell(),
                    data.stint_cell(),
                ])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use f1_term_core::stint::{Compound, Stint};

    use super::*;

    fn test_data() -> StintTableData {
        StintTableData {
            driver_tla: "ALO".to_string(),
            driver_number: DriverNumber { value: 14 },
            team_color: Color::Red,
            stints: vec![Stint {
                compound: Compound::Soft,
                lap_flags: 1,
                new: false,
                start_laps: 8,
                total_laps: 10,
                tires_not_changed: 2,
            }],
        }
    }

    #[test]
    fn test_position_cell() {
        let data = test_data();
        let cell = data.position_cell(1);
        assert_eq!(cell, Cell::new("1"));
    }

    #[test]
    fn test_tla_cell() {
        let data = test_data();
        let cell = data.tla_cell();
        assert_eq!(cell, Cell::new("1"));
        Cell::new(Span::styled("ALO", Style::default().bold().fg(Color::Red)));
    }

    #[test]
    fn test_driver_number_cell() {
        let data = test_data();
        let cell = data.driver_number_cell();
        assert_eq!(cell, Cell::new("14"));
    }
}
