use chrono::{DateTime, Datelike, Utc};
use f1_term_core::{
    clock::Clock, laps::Laps, telemetry_state::TelemetryState, track_status::TrackStatus,
    weather::Weather,
};
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
    pub grand_prix_name: String,
    pub session_name: String,
    pub circuit_name: String,
    pub country_name: String,
    pub start_date: Option<DateTime<Utc>>,
    pub weather: Weather,
    pub track_status: Option<TrackStatus>,
    pub laps: Option<Laps>,
    pub clock: Option<Clock>,
}

impl Component for TitleBar {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        if let Action::StateUpdate(ref state_lock) = action {
            let state = state_lock.read().unwrap();
            self.update_data(&state);
            return Ok(Some(Action::Render)); // render every time to update the time remaining
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
        let location_line = self.location_time_line();
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
            Span::styled(
                format!(
                    "{} {}",
                    &self.grand_prix_name,
                    self.start_date
                        .map(|d| d.year().to_string())
                        .unwrap_or("".to_string())
                ),
                Style::default().bold(),
            ),
            Span::raw(format!("  |  {} ", self.session_name))
                .dim()
                .gray(),
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
            Span::styled(" <?> ", Style::default().fg(Color::LightRed).bold()),
            Span::styled("Help  |  ", Style::default().dim()),
            Span::styled("[ ", Style::default().dim()),
            Span::styled("STATUS: ", Style::default()),
            Span::styled(
                track_status_text.to_uppercase(),
                Style::default().fg(status_color).bold(),
            ),
            Span::styled(" ]  ", Style::default().dim()),
        ])
    }

    fn location_time_line(&self) -> Line<'_> {
        // Use laps if it's there, if not fall back to time remaining on the clock
        let laps_or_time = match &self.laps {
            Some(l) => format!("Lap {}/{}", l.current, l.total),
            None => match &self.clock {
                Some(c) => format!("{}", c),
                None => "".to_string(),
            },
        };

        Line::from(vec![
            Span::raw("  "),
            Span::styled(&self.circuit_name, Style::default().bold()),
            Span::styled(format!(" ({})", self.country_name), Style::default().dim()),
            Span::raw(format!("  |  {} ", laps_or_time)).gray().dim(),
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

    fn update_data(&mut self, state: &TelemetryState) {
        if let Some(info) = &state.info {
            self.grand_prix_name.clone_from(&info.meeting.name);
            self.session_name.clone_from(&info.type_.to_string());
            self.country_name.clone_from(&info.meeting.country.name);
            self.start_date = Some(info.start_date);
        }
        if let Some(circuit) = &state.circuit {
            self.circuit_name.clone_from(&circuit.short_name);
        }
        if let Some(sw) = &state.weather
            && self.weather != *sw
        {
            self.weather.clone_from(sw);
        }
        if self.track_status != state.track_status {
            self.track_status.clone_from(&state.track_status);
        };
        if state.laps.is_some() {
            self.laps.clone_from(&state.laps);
        }
        if state.clock.is_some() {
            self.clock.clone_from(&state.clock);
        }
    }
}
