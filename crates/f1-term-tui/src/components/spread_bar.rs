use f1_term_core::gap::Gap;
use f1_term_core::lap_time::LapTime;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::canvas::{Canvas, Context, Line};

use super::Component;
use crate::action::Action;

const MIN_X: f64 = 0.0;
const MIN_Y: f64 = 0.0;
const MAX_X: f64 = 1.0;
const MAX_Y: f64 = 1.0;

const MID_Y: f64 = (MAX_Y - MIN_Y) / 2.0;

#[derive(Default)]
pub struct SpreadBar {
    drivers: Vec<DriverData>,
}

struct DriverData {
    tla: String,
    team_color: Color,
    diff_to_fastest: LapTime,
}

impl Component for SpreadBar {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::StateUpdate(ref state_lock) = action {
            let state = state_lock.read().unwrap();
            let participants = state.participants();
            self.drivers = participants
                .iter()
                .filter_map(|participant| {
                    let team = state.teams.get(&participant.driver.team_name)?;
                    let timing = state.timing_data.get(&participant.driver.number)?;
                    let session_type = state.info.as_ref().map(|info| &info.type_);

                    let diff_to_fastest = match participant.time_diff_to_fastest(session_type) {
                        Some(time_diff) => match time_diff {
                            Gap::Time(diff) => diff,
                            // Skip lapped cars (Gap::Laps) since we don't have precise values for them
                            Gap::Laps(_) => return None,
                        },
                        // Skip cars with no time value, unless they are the leader, whose gap is 0.
                        None => {
                            if timing.position == 1 {
                                LapTime::from_millis(0)
                            } else {
                                return None;
                            }
                        }
                    };
                    Some(DriverData {
                        tla: participant.driver.tla.clone(),
                        team_color: Color::from_u32(team.color.u32),
                        diff_to_fastest,
                    })
                })
                .collect();
            return Ok(Some(Action::Render));
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let canvas = Canvas::default()
            .x_bounds([MIN_X - 0.02, MAX_X + 0.02]) // small margin on the sides of the spread bar
            .y_bounds([MIN_Y, MAX_Y])
            .marker(Marker::Braille)
            .paint(|ctx| self.paint(ctx, area));
        f.render_widget(canvas, area);
        Ok(())
    }
}

impl SpreadBar {
    fn paint(&self, ctx: &mut Context, area: Rect) {
        let line = Line::new(MIN_X, MID_Y, MAX_X, MID_Y, Color::White);
        ctx.draw(&line);
        let driver_markers = self.driver_markers(area);
        for marker in driver_markers {
            let draw_x = marker.x_pos;
            let tick = Line::new(draw_x, MID_Y - 0.05, draw_x, MID_Y + 0.05, marker.color);
            ctx.draw(&tick);
            ctx.print(
                draw_x,
                marker.y_pos,
                Span::styled(marker.tla.clone(), Style::default().fg(marker.color)),
            );
        }
    }

    fn driver_markers(&self, area: Rect) -> Vec<DriverMarker> {
        let mut parsed_drivers = Vec::with_capacity(self.drivers.len());

        let mut max_diff = LapTime::from_millis(0);
        for driver in &self.drivers {
            parsed_drivers.push((driver, driver.diff_to_fastest));
            if driver.diff_to_fastest > max_diff {
                max_diff = driver.diff_to_fastest;
            }
        }

        let max_diff_millis = max_diff.millis();
        let mut driver_markers = Vec::with_capacity(self.drivers.len());
        for (driver, diff) in parsed_drivers {
            let normalized_diff = if max_diff_millis == 0 {
                1.0
            } else {
                1.0 - (diff.millis() as f64 / max_diff_millis as f64)
            };
            driver_markers.push(DriverMarker {
                x_pos: normalized_diff,
                y_pos: 0.0,
                tla: driver.tla.clone(),
                color: driver.team_color,
            });
        }

        driver_markers.sort_by(|a, b| {
            b.x_pos
                .partial_cmp(&a.x_pos)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let cell_width = (MAX_X - MIN_X) / area.width as f64;
        let cell_height = (MAX_Y - MIN_Y) / area.height as f64;
        let mut levels = [-1.0; 4];
        for marker in &mut driver_markers {
            let mut chosen_level = None;
            for (i, last_x) in levels.iter().enumerate() {
                // 3x cell width to account for TLA width
                if *last_x < 0.0 || *last_x - marker.x_pos > (cell_width * 3.0) {
                    chosen_level = Some(i);
                    break;
                }
            }

            marker.y_pos = MID_Y + cell_height;
            if let Some(l) = chosen_level {
                levels[l] = marker.x_pos;
                marker.y_pos += l as f64 * cell_height;
            }
        }

        driver_markers
    }
}

struct DriverMarker {
    x_pos: f64,
    y_pos: f64,
    tla: String,
    color: Color,
}

#[cfg(test)]
mod tests {
    use approx::relative_eq;

    use super::*;

    #[test]
    fn test_driver_markers() {
        let drivers = vec![
            DriverData {
                tla: "VER".to_string(),
                team_color: Color::Red,
                diff_to_fastest: LapTime::from_seconds(0),
            },
            DriverData {
                tla: "NOR".to_string(),
                team_color: Color::Yellow,
                diff_to_fastest: LapTime::from_seconds(10),
            },
            DriverData {
                tla: "HAM".to_string(),
                team_color: Color::Yellow,
                diff_to_fastest: LapTime::from_seconds(20),
            },
        ];

        let spread_bar = SpreadBar { drivers };
        let markers = spread_bar.driver_markers(Rect::new(0, 0, 80, 40));

        assert_eq!(markers.len(), 3);

        let ver = markers.iter().find(|m| m.tla == "VER").unwrap();
        assert_eq!(ver.x_pos, 1.0); // 0.0 diff -> 1.0
        //
        let ham = markers.iter().find(|m| m.tla == "NOR").unwrap();
        assert_eq!(ham.x_pos, 0.5); // 10.0 / 20.0 = 0.5 -> 1.0 - 0.5 = 0.5
        //
        let ham = markers.iter().find(|m| m.tla == "HAM").unwrap();
        assert_eq!(ham.x_pos, 0.0); // max diff -> 0.0
    }

    #[test]
    fn test_driver_markers_overlap_logic() {
        let drivers = vec![
            DriverData {
                tla: "VER".to_string(),
                team_color: Color::Red,
                diff_to_fastest: LapTime::from_millis(0),
            },
            DriverData {
                tla: "HAM".to_string(),
                team_color: Color::Red,
                diff_to_fastest: LapTime::from_millis(10),
            },
            DriverData {
                tla: "NOR".to_string(),
                team_color: Color::Yellow,
                diff_to_fastest: LapTime::from_millis(20),
            },
            DriverData {
                tla: "SAI".to_string(),
                team_color: Color::Blue,
                diff_to_fastest: LapTime::from_millis(30),
            },
            DriverData {
                tla: "LEC".to_string(),
                team_color: Color::Red,
                diff_to_fastest: LapTime::from_millis(40),
            },
            DriverData {
                tla: "OCO".to_string(),
                team_color: Color::Magenta,
                diff_to_fastest: LapTime::from_seconds(10), // large diff to make normalized diffs of the above 5 very close
            },
        ];

        let spread_bar = SpreadBar { drivers };
        let rect = Rect::new(0, 0, 80, 40);
        let markers = spread_bar.driver_markers(rect);

        assert_eq!(markers.len(), 6);

        let ver = markers.iter().find(|m| m.tla == "VER").unwrap();
        let ham = markers.iter().find(|m| m.tla == "HAM").unwrap();
        let nor = markers.iter().find(|m| m.tla == "NOR").unwrap();
        let sai = markers.iter().find(|m| m.tla == "SAI").unwrap();
        let lec = markers.iter().find(|m| m.tla == "LEC").unwrap();

        let cell_height = (MAX_Y - MIN_Y) / rect.height as f64;

        let _ = relative_eq!(ver.y_pos, MID_Y + cell_height);
        let _ = relative_eq!(ham.y_pos, MID_Y + 2.0 * cell_height);
        let _ = relative_eq!(nor.y_pos, MID_Y + 3.0 * cell_height);
        let _ = relative_eq!(sai.y_pos, MID_Y + 4.0 * cell_height);
        // 5th driver falls back to level 0 because 4 levels is max
        let _ = relative_eq!(lec.y_pos, MID_Y + cell_height);
    }
}
