use f1_term_core::{telemetry_state::TelemetryState, track_status::TrackStatus, weather::Weather};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::{Action, Component};

#[derive(Default)]
pub struct TitleBar {
    pub session_name: String,
    pub session_official_name: String,
    pub session_circuit_name: String,
    pub session_country_name: String,
    pub weather: Weather,
    pub track_status: Option<TrackStatus>,
}

impl Component for TitleBar {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::StateUpdate(ref state_lock) = action {
            let state = state_lock.read().unwrap();
            let updated = self.update_data(&state);
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

        let rows =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner_area);

        let title_line = self.title_line();
        let status_line = self.status_line();
        let location_line = self.location_line();
        let weather_line = self.weather_line();

        let row1_layout = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(status_line.width() as u16),
        ])
        .split(rows[0]);

        let row2_layout = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(weather_line.width() as u16),
        ])
        .split(rows[1]);

        frame.render_widget(Paragraph::new(title_line), row1_layout[0]);
        frame.render_widget(
            Paragraph::new(status_line).alignment(Alignment::Right),
            row1_layout[1],
        );

        frame.render_widget(Paragraph::new(location_line), row2_layout[0]);
        frame.render_widget(
            Paragraph::new(weather_line).alignment(Alignment::Right),
            row2_layout[1],
        );

        Ok(())
    }
}

impl TitleBar {
    fn title_line(&self) -> Line<'_> {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&self.session_official_name, Style::default().bold()),
            Span::raw(format!(" | {} ", self.session_name)).dim(),
        ])
    }

    fn status_line(&self) -> Line<'_> {
        let track_status_text = match &self.track_status {
            Some(ts) => ts.message.as_str(),
            None => "Unknown",
        };

        let status_color = if track_status_text.contains("Clear") {
            Color::Green
        } else if track_status_text.contains("Yellow") || track_status_text.contains("SC") {
            Color::Yellow
        } else if track_status_text.contains("Red") {
            Color::Red
        } else {
            Color::White
        };

        Line::from(vec![
            Span::styled("[ ", Style::default().dim()),
            Span::styled("STATUS: ", Style::default()),
            Span::styled(
                track_status_text.to_uppercase(),
                Style::default().fg(status_color).bold(),
            ),
            Span::styled(" ]  ", Style::default().dim()),
        ])
    }

    fn location_line(&self) -> Line<'_> {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(&self.session_circuit_name, Style::default().bold()),
            Span::styled(
                format!(" ({}) ", self.session_country_name),
                Style::default().dim(),
            ),
        ])
    }

    fn weather_line(&self) -> Line<'_> {
        Line::from(vec![
            Span::raw("Air: ").dim(),
            format!("{}°C   ", self.weather.air_temperature).into(),
            Span::raw("Trk: ").dim(),
            format!("{}°C   ", self.weather.track_temperature).into(),
            Span::raw("Rain: ").dim(),
            format!("{}%   ", self.weather.rainfall).into(),
            Span::raw("Wind: ").dim(),
            format!(
                "{}m/s {}   ",
                self.weather.wind.speed,
                self.weather.wind.direction.to_direction()
            )
            .bold(),
            Span::raw("Pres: ").dim(),
            format!("{}mb   ", self.weather.pressure).into(),
            Span::raw("Hum: ").dim(),
            format!("{}%  ", self.weather.humidity).into(),
        ])
    }
    fn update_data(&mut self, state: &TelemetryState) -> bool {
        let mut updated = false;
        if let Some(info) = &state.info {
            if self.session_official_name != info.meeting.official_name {
                self.session_official_name
                    .clone_from(&info.meeting.official_name);
                updated = true;
            }
            if self.session_name != info.name {
                self.session_name.clone_from(&info.name);
                updated = true;
            }
            if self.session_circuit_name != info.meeting.circuit.short_name {
                self.session_circuit_name
                    .clone_from(&info.meeting.circuit.short_name);
                updated = true;
            }
            if self.session_country_name != info.meeting.country.name {
                self.session_country_name
                    .clone_from(&info.meeting.country.name);
                updated = true;
            }
        }
        if let Some(sw) = &state.weather
            && self.weather != *sw
        {
            self.weather.clone_from(sw);
            updated = true;
        }
        if self.track_status != state.track_status {
            self.track_status.clone_from(&state.track_status);
            updated = true;
        }
        updated
    }
}
