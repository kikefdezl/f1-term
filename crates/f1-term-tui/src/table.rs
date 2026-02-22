use f1_term_core::driver::Driver;
use f1_term_core::stint::{Compound, Stints};
use f1_term_core::team::Team;
use f1_term_core::timing::{LiveTiming, SegmentStatus};
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::{Cell, Row, Table as RatatuiTable, Widget};

pub struct TableData {
    line: Option<u8>,
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
            line: args.driver.line,
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
        }
    }
}

#[derive(Default)]
pub struct Table {
    items: Vec<TableData>,
}

impl Table {
    pub fn new(items: Vec<TableData>) -> Self {
        Table { items }
    }
}

impl Widget for Table {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut items = self.items;
        items.sort_by(|a, b| {
            (a.line.is_none(), a.line, &a.driver_tla).cmp(&(
                b.line.is_none(),
                b.line,
                &b.driver_tla,
            ))
        });

        let rows = Table::_create_rows(&items);
        let header = Table::_create_header();

        let segment_len = |sector: usize| -> u16 {
            items
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
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(s1_segments),
                Constraint::Length(s2_segments),
                Constraint::Length(s3_segments),
            ],
        )
        .header(header);
        t.render(area, buf);
    }
}

impl Table {
    fn _create_rows(items: &[TableData]) -> Vec<Row<'_>> {
        let mut rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let pos = i + 1;

                let best_lap = match &data.best_lap_time {
                    Some(ll) => ll.clone(),
                    None => "-:--:---".to_string(),
                };

                let last_lap = match &data.last_lap_time {
                    Some(ll) => ll.clone(),
                    None => "-:--:---".to_string(),
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
                        .map(|s| Table::_process_segments(s))
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
                Cell::from("···").style(Style::default().fg(Color::DarkGray)),
                Cell::from("······").style(Style::default().fg(Color::DarkGray)),
                Cell::from("···").style(Style::default().fg(Color::DarkGray)),
                Cell::from("·······").style(Style::default().fg(Color::DarkGray)),
                Cell::from("········").style(Style::default().fg(Color::DarkGray)),
                Cell::from("········").style(Style::default().fg(Color::DarkGray)),
                Cell::from("··········").style(Style::default().fg(Color::DarkGray)),
                Cell::from("··········").style(Style::default().fg(Color::DarkGray)),
                Cell::from("··········").style(Style::default().fg(Color::DarkGray)),
            ]),
        );
        rows
    }

    fn _create_header() -> Row<'static> {
        Row::new(vec![
            Cell::from(Line::from("#").alignment(ratatui::layout::Alignment::Right)),
            Cell::from("Driver"),
            Cell::from("Num"),
            Cell::from("Tire"),
            Cell::from("Best Lap"),
            Cell::from("Last Lap"),
            Cell::from("S1"),
            Cell::from("S2"),
            Cell::from("S3"),
        ])
    }

    fn _process_segments(segments: &[SegmentStatus]) -> Cell<'_> {
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
                Span::styled("●", Style::default().fg(color))
            })
            .collect();

        Cell::from(Line::from(spans))
    }
}
