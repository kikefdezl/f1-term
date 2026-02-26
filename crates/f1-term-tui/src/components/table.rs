use std::sync::Arc;

use crossterm::event::KeyCode;
use f1_term_core::{
    driver::Driver,
    session::Session,
    stint::{Compound, Stints},
    team::Team,
    timing::{LiveTiming, SegmentStatus},
};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table as RatatuiTable, TableState},
};

use super::{Action, Component};

const SEGMENTS: &str = "⯀"; // other options: ▮ ▰  ● ⬤

pub struct TableData {
    driver_tla: String,
    driver_number: String,
    team_color: Color,
    tire_compound: Option<Compound>,
    tire_laps: Option<u8>,
    best_lap_time: Option<String>,
    last_lap_time: Option<String>,
    last_lap_overall_fastest: bool,
    last_lap_personal_fastest: bool,
    segments: Vec<Vec<SegmentStatus>>,
    time_diff_to_fastest: Option<String>,
    time_diff_to_position_ahead: Option<String>,
}

pub struct TableDataArgs<'a> {
    pub driver: &'a Driver,
    pub team: &'a Team,
    pub live_timing: Option<&'a LiveTiming>,
    pub stints: Option<&'a Stints>,
}

impl From<&TableDataArgs<'_>> for TableData {
    fn from(args: &'_ TableDataArgs) -> Self {
        TableData {
            driver_tla: args.driver.tla.clone(),
            driver_number: args.driver.number.value.to_string(),
            team_color: Color::from_u32(args.team.color.u32),
            tire_compound: args
                .stints
                .and_then(|s| s.last().map(|stint| stint.compound.clone())),
            tire_laps: args
                .stints
                .and_then(|s| s.last().map(|stint| stint.total_laps)),
            best_lap_time: args.live_timing.and_then(|lt| lt.best_lap_time.clone()),
            last_lap_time: args.live_timing.and_then(|lt| lt.last_lap.time.clone()),
            last_lap_overall_fastest: args
                .live_timing
                .map(|lt| lt.last_lap.overall_fastest)
                .unwrap_or(false),
            last_lap_personal_fastest: args
                .live_timing
                .map(|lt| lt.last_lap.personal_fastest)
                .unwrap_or(false),
            segments: args
                .live_timing
                .map(|lt| {
                    lt.last_lap
                        .sectors
                        .iter()
                        .map(|s| {
                            s.segments
                                .iter()
                                .map(|s| s.status.clone())
                                .collect::<Vec<SegmentStatus>>()
                        })
                        .collect()
                })
                .unwrap_or_default(),
            time_diff_to_fastest: args.live_timing.map(|lt| lt.time_diff_to_fastest.clone()),
            time_diff_to_position_ahead: args
                .live_timing
                .map(|lt| lt.time_diff_to_position_ahead.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GapMode {
    #[default]
    ToFastest,
    ToPositionAhead,
}

impl GapMode {
    pub fn toggle(&mut self) {
        *self = match self {
            GapMode::ToFastest => GapMode::ToPositionAhead,
            GapMode::ToPositionAhead => GapMode::ToFastest,
        };
    }
}

#[derive(Default)]
pub struct TableComponent {
    items: Vec<TableData>,
    state: TableState,
    gap_mode: GapMode,
    session_title: Option<String>,
}

impl TableComponent {
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn update_data(&mut self, session: &Arc<Session>) {
        let mut tds = Vec::new();
        for participant in session.leaderboard() {
            let args = TableDataArgs {
                driver: participant.driver,
                team: participant.team,
                live_timing: participant.timing,
                stints: participant.stints,
            };
            tds.push(TableData::from(&args));
        }
        self.items = tds;
    }
}

impl Component for TableComponent {
    fn update(&mut self, action: Action) -> Result<Option<Action>, Box<dyn std::error::Error>> {
        match action {
            Action::KeyPress(key) => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next();
                    return Ok(Some(Action::Render));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.previous();
                    return Ok(Some(Action::Render));
                }
                KeyCode::Char('g') => return Ok(Some(Action::ToggleGapMode)),
                _ => {}
            },
            Action::SessionUpdate(ref session) => {
                let title = format!(
                    " {} - {} | {} ({}) ",
                    session.info.meeting.official_name,
                    session.info.name,
                    session.info.meeting.circuit.short_name,
                    session.info.meeting.country.name
                );
                self.session_title = Some(title);

                if !session.drivers.is_empty() && !session.teams.is_empty() {
                    self.update_data(session);
                }
            }
            Action::ToggleGapMode => {
                self.gap_mode.toggle();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        if self.items.is_empty() {
            return Ok(());
        }

        let rows = TableComponent::_create_rows(&self.items, self.gap_mode);
        let header = TableComponent::_create_header();

        let segment_len = |sector: usize| -> u16 {
            self.items
                .first()
                .and_then(|outer| outer.segments.get(sector))
                .map_or(0, |inner| inner.len())
                .try_into()
                .expect("Should always fit in u16")
        };

        let s1_segments = segment_len(0);
        let s2_segments = segment_len(1);
        let s3_segments = segment_len(2);

        let mut block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default());

        if let Some(title) = &self.session_title {
            block = block.title(ratatui::text::Span::styled(
                title.as_str(),
                Style::default(),
            ));
        }

        let t = RatatuiTable::new(
            rows,
            [
                Constraint::Length(3),           // #
                Constraint::Length(4),           // driver
                Constraint::Length(3),           // num
                Constraint::Length(7),           // tire
                Constraint::Length(10),          // best lap
                Constraint::Length(7),           // gap
                Constraint::Length(10),          // last lap
                Constraint::Length(s1_segments), // s1
                Constraint::Length(s2_segments), // s2
                Constraint::Length(s3_segments), // s3
            ],
        )
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(t, area, &mut self.state);

        Ok(())
    }
}

impl TableComponent {
    fn _create_rows(items: &[TableData], gap_mode: GapMode) -> Vec<Row<'_>> {
        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let pos = i + 1;

                let best_lap = match &data.best_lap_time {
                    Some(ll) => ll,
                    None => "-:--.---",
                };

                let last_lap = match &data.last_lap_time {
                    Some(ll) => ll,
                    None => "-:--.---",
                };

                let time_diff = if i == 0 {
                    "------"
                } else {
                    let diff = match gap_mode {
                        GapMode::ToFastest => &data.time_diff_to_fastest,
                        GapMode::ToPositionAhead => &data.time_diff_to_position_ahead,
                    };
                    match diff {
                        Some(t) => t.as_str(),
                        None => " -.---",
                    }
                };

                let last_lap_style = if data.last_lap_overall_fastest {
                    Style::default().fg(Color::from_u32(0xBF00FF)) // #BF00FF
                } else if data.last_lap_personal_fastest {
                    Style::default().fg(Color::from_u32(0x39FF14)) // #39FF14
                } else {
                    Style::default()
                };

                let segment_data = |sector: usize| -> Cell {
                    data.segments
                        .get(sector)
                        .map(|s| TableComponent::_create_segments(s))
                        .unwrap_or_default()
                };
                let s1 = segment_data(0);
                let s2 = segment_data(1);
                let s3 = segment_data(2);

                let tire_cell = match (&data.tire_compound, data.tire_laps) {
                    (Some(compound), Some(laps)) => {
                        let (letter, color) = match compound {
                            Compound::Soft => ("S", Color::Red),
                            Compound::Medium => ("M", Color::Yellow),
                            Compound::Hard => ("H", Color::White),
                            Compound::Wet => ("W", Color::Blue),
                            Compound::Intermediate => ("I", Color::Green),
                            Compound::Unknown => ("?", Color::DarkGray),
                        };
                        Cell::from(format!("{} ({})", letter, laps))
                            .style(Style::default().fg(color))
                    }
                    _ => Cell::from(""),
                };

                Row::new(vec![
                    Cell::from(format!("{:>3}", pos)),
                    Cell::from(data.driver_tla.clone()).style(
                        Style::default()
                            .fg(data.team_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(data.driver_number.clone()),
                    tire_cell,
                    Cell::from(best_lap),
                    Cell::from(time_diff),
                    Cell::from(last_lap).style(last_lap_style),
                    s1,
                    s2,
                    s3,
                ])
            })
            .collect();
        rows
    }

    fn _create_header() -> Row<'static> {
        Row::new(vec![
            Cell::from("  #"),
            Cell::from("Drv"),
            Cell::from("Num"),
            Cell::from("Tire"),
            Cell::from("Best Lap"),
            Cell::from("Gap"),
            Cell::from("Last Lap"),
            Cell::from("S1"),
            Cell::from("S2"),
            Cell::from("S3"),
        ])
    }

    fn _create_segments(segments: &[SegmentStatus]) -> Cell<'_> {
        let spans: Vec<Span> = segments
            .iter()
            .map(|s| {
                let color = match s {
                    SegmentStatus::None => Color::DarkGray,
                    SegmentStatus::InPit => Color::Blue,
                    SegmentStatus::OverallFastest => Color::Magenta,
                    SegmentStatus::PersonalFastest => Color::Rgb(0, 255, 127), // #00FF7F
                    SegmentStatus::Normal => Color::Yellow,
                };
                Span::styled(SEGMENTS, Style::default().fg(color))
            })
            .collect();

        Cell::from(Line::from(spans))
    }
}
