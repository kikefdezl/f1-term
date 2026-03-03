use std::error::Error;

use f1_term_core::circuit::{Bounds, CircuitLayout};
use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    widgets::canvas::{Canvas, Line},
};

use super::{Action, Component};

#[derive(Default)]
pub struct CircuitCanvas {
    circuit_key: u32,
    bounds: Bounds,
    segments: Vec<Line>,
}

impl Component for CircuitCanvas {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn Error>> {
        if let Action::StateUpdate(state_lock) = &action {
            let state = state_lock.read().unwrap();
            if let Some(info) = state.info.as_ref()
                && let Some(layout) = &info.meeting.circuit.layout
                && (self.circuit_key != info.meeting.circuit.key || self.segments.is_empty())
            {
                self.circuit_key = info.meeting.circuit.key;
                self.bounds = layout.bounds();
                self.segments = segments_from_layout(layout);
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
            })
            .x_bounds([self.bounds.x_min as f64, self.bounds.x_max as f64])
            .y_bounds([self.bounds.y_min as f64, self.bounds.y_max as f64]);

        frame.render_widget(canvas, area);
        Ok(())
    }
}

fn segments_from_layout(layout: &CircuitLayout) -> Vec<Line> {
    let mut lines = Vec::new();
    let (x_rot, y_rot) = layout.rotated_points();
    for i in 0..x_rot.len().saturating_sub(1) {
        lines.push(Line::new(
            x_rot[i],
            y_rot[i],
            x_rot[i + 1],
            y_rot[i + 1],
            Color::White,
        ));
    }
    lines
}
