use std::error::Error;

use crossterm::event::KeyCode;
use f1_term_core::circuit::{
    Bounds, CircuitKey, CircuitLayout, CircuitScope, CircuitStatus, Corner,
};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Canvas, Line};

use super::{Action, Component};

const CIRCUIT_THICKNESS: f64 = 0.5;

#[derive(Default)]
pub struct CircuitCanvas {
    circuit_key: CircuitKey,
    circuit_status: CircuitStatus,
    bounds: Bounds,
    segments: Vec<Line>,
    show_corners: bool,
    corners: Vec<Corner>,
}

impl Component for CircuitCanvas {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn Error>> {
        match &action {
            Action::StateUpdate(state_lock) => {
                let state = state_lock.read().unwrap();
                if let Some(circuit) = &state.circuit
                    && let Some(layout) = &circuit.layout
                    && (self.circuit_key != circuit.key
                        || self.circuit_status != circuit.status
                        || self.segments.is_empty())
                {
                    let layout = layout.rotate();
                    self.circuit_key = circuit.key;
                    self.circuit_status = circuit.status.clone();
                    self.bounds = layout.bounds();
                    self.corners = layout.corners.clone();
                    self.segments = segments_from_layout(&layout, &self.circuit_status);
                }
            }
            Action::KeyPress(key) => {
                if let KeyCode::Char('n') = key.code {
                    self.toggle_show_curve_numbers();
                }
                return Ok(Some(Action::Render));
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn Error>> {
        if self.segments.is_empty() {
            return Ok(());
        }

        let bounds = pad_bounds_to_area(&self.bounds, area);

        let canvas = Canvas::default()
            .marker(ratatui::symbols::Marker::Braille)
            .paint(|ctx| self.draw_circuit(ctx, bounds, area))
            .x_bounds([bounds.x_min as f64, bounds.x_max as f64])
            .y_bounds([bounds.y_min as f64, bounds.y_max as f64]);

        frame.render_widget(canvas, area);
        Ok(())
    }
}

impl CircuitCanvas {
    /// Draws the circuit layout on the canvas.
    ///
    /// To create a simulated line thickness, we draw each segment multiple times,
    /// slightly offset in the X and Y coordinate spaces.
    /// This effectively creates a thicker "brush" around the true coordinate path.
    fn draw_circuit(
        &self,
        ctx: &mut ratatui::widgets::canvas::Context<'_>,
        bounds: Bounds,
        area: Rect,
    ) {
        // This calculates the coordinate size of a single braille dot.
        let dot_size_x = if area.width > 0 {
            (bounds.x_max - bounds.x_min) as f64 / (area.width as f64 * 2.0)
        } else {
            1.0
        };
        let dot_size_y = if area.height > 0 {
            (bounds.y_max - bounds.y_min) as f64 / (area.height as f64 * 4.0)
        } else {
            1.0
        };

        let dx = dot_size_x * CIRCUIT_THICKNESS;
        let dy = dot_size_y * CIRCUIT_THICKNESS;

        let offsets = [(0.0, 0.0), (0.0, dy), (0.0, -dy), (dx, 0.0), (-dx, 0.0)];

        for segment in &self.segments {
            for (ox, oy) in offsets {
                ctx.draw(&Line {
                    x1: segment.x1 + ox,
                    y1: segment.y1 + oy,
                    x2: segment.x2 + ox,
                    y2: segment.y2 + oy,
                    color: segment.color,
                });
            }
        }

        if self.show_corners {
            for corner in &self.corners {
                ctx.print(
                    corner.coord.x,
                    corner.coord.y,
                    ratatui::text::Span::styled(
                        format!("{}", corner.num),
                        ratatui::style::Style::default().fg(Color::White),
                    ),
                );
            }
        }
    }

    fn toggle_show_curve_numbers(&mut self) {
        self.show_corners = !self.show_corners
    }
}

fn segments_from_layout(layout: &CircuitLayout, status: &CircuitStatus) -> Vec<Line> {
    let mut lines = Vec::new();

    for i in 0..layout.coords.len().saturating_sub(1) {
        let mut color = Color::White;

        match status {
            CircuitStatus::Clear => color = Color::White,
            CircuitStatus::Red => color = Color::Red,
            CircuitStatus::Yellow(CircuitScope::Full) => color = Color::Yellow,
            CircuitStatus::Yellow(CircuitScope::Sectors(sectors)) => {
                for &sector in sectors {
                    let ms_idx = sector.saturating_sub(1) as usize;
                    if let Some(mini_sectors) = &layout.mini_sectors
                        && let Some(range) = mini_sectors.get(ms_idx)
                        && range.contains(&i)
                    {
                        color = Color::Yellow;
                        break;
                    }
                }
            }
        }

        lines.push(Line::new(
            layout.coords[i].x,
            layout.coords[i].y,
            layout.coords[i + 1].x,
            layout.coords[i + 1].y,
            color,
        ));
    }

    lines
}

/// Ratatui canvas widgets stretches the content to fit the area, so we add
/// some padding to the bounds to make sure the circuit has the same shape
/// as the real layout with minimal distortion.
fn pad_bounds_to_area(bounds: &Bounds, area: Rect) -> Bounds {
    // Terminal cells are usually a 1:2 aspect ratio
    // We need to account for this to avoid distortion.
    let grid_width = area.width as f32 * 1.0;
    let grid_height = area.height as f32 * 2.0;

    let width = (bounds.x_max - bounds.x_min) as f32;
    let height = (bounds.y_max - bounds.y_min) as f32;

    let scale_x = grid_width / width;
    let scale_y = grid_height / height;

    if scale_x < scale_y {
        // Circuit is horizontal, pad the height
        let new_height = grid_height / scale_x;
        let buffer_y = (new_height - height) / 2.0;
        Bounds {
            x_min: bounds.x_min,
            y_min: (bounds.y_min as f32 - (2.0 * buffer_y)).round() as i32,
            x_max: bounds.x_max,
            y_max: bounds.y_max,
        }
    } else {
        // Circuit is vertical, Pad the width
        let new_width = grid_width / scale_y;
        let buffer_x = (new_width - width) / 2.0;
        Bounds {
            x_min: (bounds.x_min as f32 - buffer_x).round() as i32,
            y_min: bounds.y_min,
            x_max: (bounds.x_max as f32 + buffer_x).round() as i32,
            y_max: bounds.y_max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_bounds_to_area_no_padding_needed() {
        // Grid is 100x100 dots (50 * 2, 25 * 4), matching bounds 100x100
        let bounds = Bounds {
            x_min: 0,
            y_min: 0,
            x_max: 100,
            y_max: 100,
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 25,
        };
        let padded = pad_bounds_to_area(&bounds, area);

        assert_eq!(padded.x_min, 0);
        assert_eq!(padded.x_max, 100);
        assert_eq!(padded.y_min, 0);
        assert_eq!(padded.y_max, 100);
    }

    #[test]
    fn test_pad_bounds_to_area_widen_horizontally() {
        // Grid is 200x100 dots (100 * 2, 25 * 4), meaning area is wider.
        // We need to expand bounds X to cover 200.
        let bounds = Bounds {
            x_min: 0,
            y_min: 0,
            x_max: 100,
            y_max: 100,
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 25,
        };
        let padded = pad_bounds_to_area(&bounds, area);

        assert_eq!(padded.x_min, -50);
        assert_eq!(padded.x_max, 150);
        assert_eq!(padded.y_min, 0);
        assert_eq!(padded.y_max, 100);
    }

    #[test]
    fn test_pad_bounds_to_area_widen_vertically() {
        // Grid is 100x200 dots (50 * 2, 50 * 4), meaning area is taller.
        // We need to expand bounds Y to cover 200.
        let bounds = Bounds {
            x_min: 0,
            y_min: 0,
            x_max: 100,
            y_max: 100,
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 50,
        };
        let padded = pad_bounds_to_area(&bounds, area);

        assert_eq!(padded.x_min, 0);
        assert_eq!(padded.x_max, 100);
        assert_eq!(padded.y_min, -100);
        assert_eq!(padded.y_max, 100);
    }
}
