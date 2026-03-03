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
    for i in 0..layout.x.len().saturating_sub(1) {
        lines.push(Line::new(
            layout.x[i] as f64,
            layout.y[i] as f64,
            layout.x[i + 1] as f64,
            layout.y[i + 1] as f64,
            Color::White,
        ));
    }
    lines
}
