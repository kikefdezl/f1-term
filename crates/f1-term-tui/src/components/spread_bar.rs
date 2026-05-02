use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::canvas::{Canvas, Context, Line};

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
    position: u8,
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
                    let diff_to_fastest = participant.time_diff_to_fastest(session_type);
                    let position = timing.position;
                    Some(DriverData {
                        tla: participant.driver.tla.clone(),
                        team_color: Color::from_u32(team.color.u32),
                        diff_to_fastest,
                        position,
                    })
                })
                .collect();
            return Ok(Some(Action::Render));
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let canvas = Canvas::default()
            .x_bounds([-0.02, 1.02]) // small margin on the sides of the spread bar
            .y_bounds([0.0, 1.0])
            .marker(Marker::Braille)
            .paint(|ctx| self.paint(ctx));
        f.render_widget(canvas, area);
        Ok(())
    }
}

impl SpreadBar {
    fn paint(&self, ctx: &mut Context) {
        let line = Line::new(0.0, 0.5, 1.0, 0.5, Color::White);
        ctx.draw(&line);
        let driver_markers = self.driver_markers();
        for marker in driver_markers {
            let draw_x = marker.normalized_diff;
            let tick = Line::new(draw_x, 0.45, draw_x, 0.55, marker.color);
            ctx.draw(&tick);
            ctx.print(
                draw_x,
                marker.y_offset,
                Span::styled(marker.tla.clone(), Style::default().fg(marker.color)),
            );
        }
    }

    fn driver_markers(&self) -> Vec<DriverMarker> {
        let mut parsed_drivers = Vec::new();
        let mut max_diff = 0.0_f64;

        for driver in &self.drivers {
            let diff = if driver.position == 1 {
                Some(0.0)
            } else {
                let diff_str = driver.diff_to_fastest.as_deref().unwrap_or("");
                Self::parse_diff(diff_str)
            };

            if let Some(d) = diff {
                if d > max_diff {
                    max_diff = d;
                }
                parsed_drivers.push((driver, d));
            }
        }

        let mut driver_markers = Vec::new();
        for (driver, diff) in parsed_drivers {
            let normalized_diff = if max_diff == 0.0 {
                1.0
            } else {
                1.0 - (diff / max_diff)
            };
            driver_markers.push(DriverMarker {
                normalized_diff,
                tla: driver.tla.clone(),
                color: driver.team_color,
                y_offset: 1.0,
            });
        }

        driver_markers.sort_by(|a, b| {
            b.normalized_diff
                .partial_cmp(&a.normalized_diff)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut levels = [-1.0; 4];
        for marker in &mut driver_markers {
            let mut chosen_level = None;
            for (i, last_x) in levels.iter().enumerate() {
                if *last_x < 0.0 || *last_x - marker.normalized_diff > 0.05 {
                    chosen_level = Some(i);
                    break;
                }
            }

            marker.y_offset = 0.6;
            if let Some(l) = chosen_level {
                levels[l] = marker.normalized_diff;
                marker.y_offset += l as f64 * 0.05;
            }
        }

        driver_markers
    }

    fn parse_diff(diff: &str) -> Option<f64> {
        if diff.is_empty() {
            return None;
        }

        if diff.ends_with('L') {
            return None;
        }

        let diff = diff.trim_start_matches('+');

        if diff.contains(':') {
            let parts: Vec<&str> = diff.split(':').collect();
            if parts.len() == 2 {
                let minutes = parts[0].parse::<f64>().ok()?;
                let seconds = parts[1].parse::<f64>().ok()?;
                Some(minutes * 60.0 + seconds)
            } else {
                None
            }
        } else {
            diff.parse::<f64>().ok()
        }
    }
}

struct DriverMarker {
    normalized_diff: f64,
    tla: String,
    color: Color,
    y_offset: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff() {
        assert_eq!(SpreadBar::parse_diff("+1.234"), Some(1.234));
        assert_eq!(SpreadBar::parse_diff("1.234"), Some(1.234));
        assert_eq!(SpreadBar::parse_diff("+1:23.456"), Some(83.456));
        assert_eq!(SpreadBar::parse_diff("1:23.456"), Some(83.456));
        assert_eq!(SpreadBar::parse_diff("0.000"), Some(0.0));
        assert_eq!(SpreadBar::parse_diff("1L"), None);
        assert_eq!(SpreadBar::parse_diff("+2L"), None);
        assert_eq!(SpreadBar::parse_diff("invalid"), None);
        assert_eq!(SpreadBar::parse_diff(""), None);
    }

    #[test]
    fn test_driver_markers() {
        let drivers = vec![
            DriverData {
                tla: "VER".to_string(),
                team_color: Color::Blue,
                diff_to_fastest: Some("".to_string()),
                position: 1,
            },
            DriverData {
                tla: "HAM".to_string(),
                team_color: Color::Green,
                diff_to_fastest: Some("+10.0".to_string()),
                position: 2,
            },
            DriverData {
                tla: "NOR".to_string(),
                team_color: Color::Yellow,
                diff_to_fastest: Some("+20.0".to_string()),
                position: 3,
            },
            DriverData {
                tla: "SAI".to_string(),
                team_color: Color::Red,
                diff_to_fastest: Some("1L".to_string()), // Should be ignored
                position: 4,
            },
        ];

        let spread_bar = SpreadBar { drivers };
        let markers = spread_bar.driver_markers();

        assert_eq!(markers.len(), 3);

        let ver = markers.iter().find(|m| m.tla == "VER").unwrap();
        assert_eq!(ver.normalized_diff, 1.0); // 0.0 diff -> 1.0

        let ham = markers.iter().find(|m| m.tla == "HAM").unwrap();
        assert_eq!(ham.normalized_diff, 0.5); // 10.0 / 20.0 = 0.5 -> 1.0 - 0.5 = 0.5

        let nor = markers.iter().find(|m| m.tla == "NOR").unwrap();
        assert_eq!(nor.normalized_diff, 0.0); // 20.0 / 20.0 = 1.0 -> 1.0 - 1.0 = 0.0
    }

    #[test]
    fn test_driver_markers_empty_or_none() {
        let drivers = vec![
            DriverData {
                tla: "VER".to_string(),
                team_color: Color::Blue,
                diff_to_fastest: None,
                position: 1,
            },
            DriverData {
                tla: "HAM".to_string(),
                team_color: Color::Green,
                diff_to_fastest: Some("".to_string()),
                position: 2, // Should be ignored because empty and not pos 1
            },
            DriverData {
                tla: "NOR".to_string(),
                team_color: Color::Yellow,
                diff_to_fastest: Some("0.000".to_string()),
                position: 1,
            },
            DriverData {
                tla: "SAI".to_string(),
                team_color: Color::Red,
                diff_to_fastest: Some("GARBAGE".to_string()),
                position: 1, // Should be forced to 0.0 diff and included
            },
        ];

        let spread_bar = SpreadBar { drivers };
        let markers = spread_bar.driver_markers();

        // VER, NOR, and SAI should be included because they have position 1
        assert_eq!(markers.len(), 3);

        // All are 0.0 diff, so all should have 1.0 normalized_diff
        for marker in markers {
            assert_eq!(marker.normalized_diff, 1.0);
        }
    }

    #[test]
    fn test_driver_markers_overlap_logic() {
        // Create 5+ DriverData items clustered very closely
        let drivers = vec![
            DriverData {
                tla: "VER".to_string(),
                team_color: Color::Blue,
                diff_to_fastest: None,
                position: 1, // diff 0.0 -> normalized 1.0
            },
            DriverData {
                tla: "HAM".to_string(),
                team_color: Color::Green,
                diff_to_fastest: Some("+0.01".to_string()),
                position: 2,
            },
            DriverData {
                tla: "NOR".to_string(),
                team_color: Color::Yellow,
                diff_to_fastest: Some("+0.02".to_string()),
                position: 3,
            },
            DriverData {
                tla: "SAI".to_string(),
                team_color: Color::Red,
                diff_to_fastest: Some("+0.03".to_string()),
                position: 4,
            },
            DriverData {
                tla: "LEC".to_string(),
                team_color: Color::Red,
                diff_to_fastest: Some("+0.04".to_string()),
                position: 5,
            },
            DriverData {
                tla: "ALO".to_string(),
                team_color: Color::Green,
                diff_to_fastest: Some("+10.0".to_string()), // large diff to make normalized diffs of the above 5 very close
                position: 6,
            },
        ];

        let spread_bar = SpreadBar { drivers };
        let markers = spread_bar.driver_markers();

        assert_eq!(markers.len(), 6);

        let ver = markers.iter().find(|m| m.tla == "VER").unwrap();
        let ham = markers.iter().find(|m| m.tla == "HAM").unwrap();
        let nor = markers.iter().find(|m| m.tla == "NOR").unwrap();
        let sai = markers.iter().find(|m| m.tla == "SAI").unwrap();
        let lec = markers.iter().find(|m| m.tla == "LEC").unwrap();

        // 1st driver
        assert_eq!(ver.y_offset, 0.6);
        // 2nd driver
        assert_eq!(ham.y_offset, 0.65);
        // 3rd driver
        assert_eq!(nor.y_offset, 0.70);
        // 4th driver
        assert_eq!(sai.y_offset, 0.75);
        // 5th driver falls back to level 0 because 4 levels is max
        assert_eq!(lec.y_offset, 0.6);
    }
}
