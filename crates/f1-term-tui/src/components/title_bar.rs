use std::sync::Arc;

use f1_term_core::{session::Session, track_status::TrackStatus, weather::Weather};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::{Action, Component};

#[derive(Default)]
pub struct TitleBar {
    pub session_type: String,
    pub session_name: String,
    pub session_official_name: String,
    pub session_circuit_name: String,
    pub session_country_name: String,
    pub weather: Weather,
    pub track_status: Option<TrackStatus>,
}

impl Component for TitleBar {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::SessionUpdate(ref session) = action {
            let updated = self.update_data(session);
            if updated {
                return Ok(Some(Action::Render));
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default());

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let title = format!(
            "  {} - {} {} | {} ({}) ",
            self.session_official_name,
            self.session_type,
            self.session_name,
            self.session_circuit_name,
            self.session_country_name,
        );

        let title_span = Span::from(title).bold();

        let weather_text = format!(
            "🌡air: {}°C  🌡 track: {}°C   🌧: {}%   ༄: {}m/s {}   Ψ: {}mb   🌢: {}%",
            self.weather.air_temperature,
            self.weather.track_temperature,
            self.weather.rainfall,
            self.weather.wind.speed,
            self.weather.wind.direction.to_direction(),
            self.weather.pressure,
            self.weather.humidity,
        );
        let weather_span = Span::from(weather_text);

        let track_status_text = match &self.track_status {
            Some(ts) => ts.message.as_str(),
            None => "Track Status Unknown",
        };
        let track_status_span = Span::from(track_status_text);

        let layout = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Min(0),
            Constraint::Length(track_status_text.len() as u16 + 2), // +2 for right padding
        ])
        .split(inner_area);

        let title_para = Paragraph::new(vec![Line::from(""), Line::from(title_span)]);
        let weather_para = Paragraph::new(vec![Line::from(""), Line::from(weather_span)]);
        let status_para = Paragraph::new(vec![Line::from(""), Line::from(track_status_span)])
            .alignment(Alignment::Right);

        frame.render_widget(title_para, layout[0]);
        frame.render_widget(weather_para, layout[1]);
        frame.render_widget(status_para, layout[2]);

        Ok(())
    }
}

impl TitleBar {
    fn update_data(&mut self, session: &Arc<Session>) -> bool {
        let mut updated = false;
        if self.session_official_name != session.info.meeting.official_name {
            self.session_official_name = session.info.meeting.official_name.clone();
            updated = true;
        }
        if self.session_type != session.info.type_.to_string() {
            self.session_type = session.info.type_.to_string();
            updated = true;
        }
        if self.session_name != session.info.name {
            self.session_name = session.info.name.clone();
            updated = true;
        }
        if self.session_circuit_name != session.info.meeting.circuit.short_name {
            self.session_circuit_name = session.info.meeting.circuit.short_name.clone();
            updated = true;
        }
        if self.session_country_name != session.info.meeting.country.name {
            self.session_country_name = session.info.meeting.country.name.clone();
            updated = true;
        }
        if let Some(sw) = &session.weather
            && self.weather != *sw
        {
            self.weather = sw.clone();
            updated = true;
        }
        if self.track_status != session.track_status {
            self.track_status = session.track_status.clone();
            updated = true;
        }
        updated
    }
}
