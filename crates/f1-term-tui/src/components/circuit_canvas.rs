use std::error::Error;

use f1_term_core::circuit::{Bounds, CircuitLayout, Corner};
use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    text::Text,
    widgets::canvas::{Canvas, Line},
};

use super::{Action, Component};

#[derive(Default)]
pub struct CircuitCanvas {
    circuit_key: u32,
    bounds: Bounds,
    segments: Vec<Line>,
    corners: Vec<Corner>,
}

impl Component for CircuitCanvas {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn Error>> {
        if let Action::StateUpdate(state_lock) = &action {
            let state = state_lock.read().unwrap();
            if let Some(info) = state.info.as_ref()
                && let Some(layout) = &info.meeting.circuit.layout
                && (self.circuit_key != info.meeting.circuit.key || self.segments.is_empty())
            {
                let rotated = layout.rotate();
                self.circuit_key = info.meeting.circuit.key;
                self.bounds = rotated.bounds();
                self.corners = rotated.corners.clone();
                self.segments = segments_from_layout(&rotated);
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn Error>> {
        if self.segments.is_empty() {
            return Ok(());
        }

        let canvas = Canvas::default()
            .marker(ratatui::symbols::Marker::Braille)
            .paint(|ctx| {
                for segment in &self.segments {
                    ctx.draw(segment);
                }
                for corner in &self.corners {
                    ctx.print(corner.coord.x, corner.coord.y, format!("{}", corner.num));
                }
            })
            .x_bounds([self.bounds.x_min as f64, self.bounds.x_max as f64])
            .y_bounds([self.bounds.y_min as f64, self.bounds.y_max as f64]);

        frame.render_widget(canvas, area);
        Ok(())
    }
}

fn segments_from_layout(layout: &CircuitLayout) -> Vec<Line> {
    let mut lines = Vec::new();
    for i in 0..layout.coords.len().saturating_sub(1) {
        lines.push(Line::new(
            layout.coords[i].x,
            layout.coords[i].y,
            layout.coords[i + 1].x,
            layout.coords[i + 1].y,
            Color::White,
        ));
    }
    lines
}
