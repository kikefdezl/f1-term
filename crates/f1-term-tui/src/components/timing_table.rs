use crossterm::event::KeyCode;
use f1_term_core::{
    driver::Driver,
    stint::{Compound, Stints},
    team::Team,
    telemetry_state::TelemetryState,
    timing::{LiveTiming, Sector, Segment, SegmentStatus},
};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table as RatatuiTable, TableState},
};

use super::{Action, Component};

const SEGMENTS: &str = "▰ "; // other options: ▮ ▰  ● ⬤
const SEGMENT_WIDTH: u16 = 2; // The render with of the character above

const COLOR_OVERALL_FASTEST: Color = Color::from_u32(0xB11DFB); // #B11DFB
const COLOR_PERSONAL_FASTEST: Color = Color::from_u32(0x33D176); // #33D176
const COLOR_SLOWER: Color = Color::Yellow;

#[derive(Default)]
pub struct TimingTableData {
    driver_tla: String,
    driver_number: String,
    team_color: Color,
    tire_compound: Option<Compound>,
    tire_laps: Option<u8>,
    in_pit: bool,
    best_lap_time: Option<String>,
    last_lap_time: Option<String>,
    last_lap_overall_fastest: bool,
    last_lap_personal_fastest: bool,
    sectors: Vec<Sector>,
    time_diff_to_fastest: Option<String>,
    time_diff_to_position_ahead: Option<String>,
}

pub struct TimingTableDataArgs<'a> {
    pub driver: &'a Driver,
    pub team: &'a Team,
    pub live_timing: Option<&'a LiveTiming>,
    pub stints: Option<&'a Stints>,
}

impl TimingTableData {
    fn update_from(&mut self, args: &TimingTableDataArgs<'_>) {
        self.driver_tla.clone_from(&args.driver.tla);
        self.driver_number = args.driver.number.value.to_string();
        self.team_color = Color::from_u32(args.team.color.u32);

        let last_stint = args.stints.and_then(|s| s.last());
        if let Some(stint) = last_stint {
            self.tire_compound = Some(stint.compound.clone());
            self.tire_laps = Some(stint.total_laps);
        } else {
            self.tire_compound = None;
            self.tire_laps = None;
        }

        self.in_pit = args.live_timing.map(|lt| lt.in_pit).unwrap_or(false);

        self.best_lap_time
            .clone_from(&args.live_timing.and_then(|lt| lt.best_lap_time.clone()));
        self.last_lap_time
            .clone_from(&args.live_timing.and_then(|lt| lt.last_lap.time.clone()));

        self.last_lap_overall_fastest = args
            .live_timing
            .map(|lt| lt.last_lap.overall_fastest)
            .unwrap_or(false);
        self.last_lap_personal_fastest = args
            .live_timing
            .map(|lt| lt.last_lap.personal_fastest)
            .unwrap_or(false);

        if let Some(lt) = args.live_timing {
            self.sectors.clone_from(&lt.last_lap.sectors);
        } else {
            self.sectors.clear();
        }

        self.time_diff_to_fastest.clone_from(
            &args
                .live_timing
                .and_then(|lt| lt.time_diff_to_fastest.clone()),
        );
        self.time_diff_to_position_ahead.clone_from(
            &args
                .live_timing
                .and_then(|lt| lt.time_diff_to_position_ahead.clone()),
        );
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
pub struct TimingTable {
    items: Vec<TimingTableData>,
    state: TableState,
    gap_mode: GapMode,
}

impl TimingTable {
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

    pub fn deselect(&mut self) {
        self.state.select(None);
    }

    fn update_data(&mut self, state: &TelemetryState) {
        let leaderboard = state.leaderboard();

        if self.items.len() < leaderboard.len() {
            self.items
                .resize_with(leaderboard.len(), TimingTableData::default);
        } else {
            self.items.truncate(leaderboard.len());
        }

        for (i, participant) in leaderboard.into_iter().enumerate() {
            let args = TimingTableDataArgs {
                driver: participant.driver,
                team: participant.team,
                live_timing: participant.timing,
                stints: participant.stints,
            };
            self.items[i].update_from(&args);
        }
    }
}

impl Component for TimingTable {
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
                KeyCode::Char('g') => {
                    self.gap_mode.toggle();
                    return Ok(Some(Action::Render));
                }
                KeyCode::Esc => {
                    self.deselect();
                    return Ok(Some(Action::Render));
                }
                _ => {}
            },
            Action::StateUpdate(ref state_lock) => {
                let state = state_lock.read().unwrap();
                if !state.drivers.is_empty() && !state.teams.is_empty() {
                    self.update_data(&state);
                    return Ok(Some(Action::Render));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), Box<dyn std::error::Error>> {
        if self.items.is_empty() {
            return Ok(());
        }

        let rows = TimingTable::rows(&self.items, self.gap_mode);
        let header = TimingTable::header();

        let segment_len = |sector: usize| -> u16 {
            self.items
                .first()
                .and_then(|td| td.sectors.get(sector))
                .map_or(0, |inner| inner.segments.len())
                .try_into()
                .expect("Should always fit in u16")
        };

        let s1_segments = segment_len(0) * SEGMENT_WIDTH;
        let s2_segments = segment_len(1) * SEGMENT_WIDTH;
        let s3_segments = segment_len(2) * SEGMENT_WIDTH;

        let t = RatatuiTable::new(
            rows,
            [
                Constraint::Length(3),           // #
                Constraint::Length(4),           // driver
                Constraint::Length(3),           // num
                Constraint::Length(7),           // tire
                Constraint::Length(4),           // pit
                Constraint::Length(10),          // best lap
                Constraint::Length(8),           // gap
                Constraint::Length(11),          // last lap
                Constraint::Length(s1_segments), // s1 segments
                Constraint::Length(9),           // s1
                Constraint::Length(s2_segments), // s2 segments
                Constraint::Length(9),           // s2
                Constraint::Length(s3_segments), // s3 segments
                Constraint::Length(9),           // s3
            ],
        )
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(t, area, &mut self.state);

        Ok(())
    }
}

impl TimingTable {
    fn header() -> Row<'static> {
        Row::new(vec![
            Cell::from("  #"),
            Cell::from("Drv"),
            Cell::from("Num"),
            Cell::from("Tire"),
            Cell::from("Pit"),
            Cell::from("Best Lap"),
            Cell::from("Gap"),
            Cell::from("Last Lap"),
            Cell::from("S1"),
            Cell::from(""),
            Cell::from("S2"),
            Cell::from(""),
            Cell::from("S3"),
            Cell::from(""),
        ])
    }
    fn rows(items: &[TimingTableData], gap_mode: GapMode) -> Vec<Row<'_>> {
        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let pos = i + 1;

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

                let pit_cell = match data.in_pit {
                    true => Cell::from("Pit").style(Style::default().fg(Color::Blue)),
                    false => Cell::from(""),
                };

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
                    Style::default().fg(COLOR_OVERALL_FASTEST)
                } else if data.last_lap_personal_fastest {
                    Style::default().fg(COLOR_PERSONAL_FASTEST)
                } else {
                    Style::default().fg(COLOR_SLOWER)
                };

                let sector_data = |sector: usize| -> Cell {
                    data.sectors
                        .get(sector)
                        .map(|s| TimingTable::sector(s))
                        .unwrap_or(Cell::from(""))
                };
                let s1 = sector_data(0);
                let s2 = sector_data(1);
                let s3 = sector_data(2);

                let segment_data = |sector: usize| -> Cell {
                    data.sectors
                        .get(sector)
                        .map(|s| TimingTable::segments(&s.segments))
                        .unwrap_or_default()
                };
                let s1_segments = segment_data(0);
                let s2_segments = segment_data(1);
                let s3_segments = segment_data(2);

                Row::new(vec![
                    Cell::from(format!("{:>3}", pos)),
                    Cell::from(data.driver_tla.clone()).style(
                        Style::default()
                            .fg(data.team_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(data.driver_number.as_str()),
                    tire_cell,
                    pit_cell,
                    Cell::from(best_lap),
                    Cell::from(time_diff),
                    Cell::from(last_lap).style(last_lap_style),
                    s1_segments,
                    s1,
                    s2_segments,
                    s2,
                    s3_segments,
                    s3,
                ])
            })
            .collect();
        rows
    }

    fn sector(sector: &Sector) -> Cell<'_> {
        let value = match &sector.value {
            Some(v) => v,
            None => match &sector.previous_value {
                Some(pv) => pv.as_str(),
                None => "",
            },
        };

        let color = if sector.value.is_none() {
            Color::DarkGray
        } else if sector.overall_fastest {
            COLOR_OVERALL_FASTEST
        } else if sector.personal_fastest {
            COLOR_PERSONAL_FASTEST
        } else {
            COLOR_SLOWER
        };

        Cell::from(value).style(Style::default().fg(color))
    }

    fn segments(segments: &[Segment]) -> Cell<'_> {
        let spans: Vec<Span> = segments
            .iter()
            .map(|s| {
                let color = match s.status {
                    SegmentStatus::None => Color::DarkGray,
                    SegmentStatus::InPit => Color::Blue,
                    SegmentStatus::OverallFastest => COLOR_OVERALL_FASTEST,
                    SegmentStatus::PersonalFastest => COLOR_PERSONAL_FASTEST,
                    SegmentStatus::Aborted => Color::Red,
                    SegmentStatus::Normal => COLOR_SLOWER,
                };
                Span::styled(SEGMENTS, Style::default().fg(color))
            })
            .collect();

        Cell::from(Line::from(spans))
    }
}
