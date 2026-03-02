use std::error::Error;

use f1_term_core::circuit::CircuitLayout;
use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    widgets::canvas::{Canvas, Line},
};

use super::{Action, Component};

#[derive(Default)]
pub struct CircuitCanvas {
    layout: Option<CircuitLayout>,
}

impl Component for CircuitCanvas {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn Error>> {
        if let Action::StateUpdate(state_lock) = &action {
            let state = state_lock.read().unwrap();
            if let Some(info) = state.info.as_ref() {
                self.layout.clone_from(&info.meeting.circuit.layout);
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn Error>> {
        let bounds = self.layout.as_ref().map(|l| l.bounds()).unwrap_or_default();
        let canvas = Canvas::default()
            .marker(ratatui::symbols::Marker::Braille)
            .paint(|ctx| {
                if let Some(layout) = &self.layout {
                    let segments = segments_from_layout(layout);
                    for segment in &segments {
                        ctx.draw(segment);
                    }
                }
            })
            .x_bounds([bounds.x_min as f64, bounds.x_max as f64])
            .y_bounds([bounds.y_min as f64, bounds.y_max as f64]);
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
