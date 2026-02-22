use f1_term_core::{
    driver::Driver,
    stint::{Compound, Stints},
    team::Team,
    timing::{LiveTiming, SegmentStatus},
};
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Row, Table as RatatuiTable, Widget},
};

use super::state::GapMode;

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

impl TableData {
    pub fn from(args: &TableDataArgs) -> Self {
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

pub struct Table {
    items: Vec<TableData>,
    gap_mode: GapMode,
}

impl Table {
    pub fn new(items: Vec<TableData>, gap_mode: GapMode) -> Self {
        Table { items, gap_mode }
    }
}

impl Widget for Table {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rows = self._create_rows();
        let header = Table::_create_header();

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
        .header(header);
        t.render(area, buf);
    }
}

impl Table {
    fn _create_rows(&self) -> Vec<Row<'_>> {
        let mut rows: Vec<Row> = self
            .items
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
                    let diff = match self.gap_mode {
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
                        .map(|s| Table::_create_segments(s))
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

        rows.insert(
            0,
            Row::new(vec![
                Cell::from("···").style(Style::default().fg(Color::DarkGray)), // #
                Cell::from("····").style(Style::default().fg(Color::DarkGray)), // Driver
                Cell::from("···").style(Style::default().fg(Color::DarkGray)), // Num
                Cell::from("·······").style(Style::default().fg(Color::DarkGray)), // Tire
                Cell::from("·········").style(Style::default().fg(Color::DarkGray)), // Best Lap
                Cell::from("·······").style(Style::default().fg(Color::DarkGray)), // Gap
                Cell::from("·········").style(Style::default().fg(Color::DarkGray)), // Last Lap
                Cell::from("············").style(Style::default().fg(Color::DarkGray)), // S1
                Cell::from("············").style(Style::default().fg(Color::DarkGray)), // S2
                Cell::from("············").style(Style::default().fg(Color::DarkGray)), // S3
            ]),
        );
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
