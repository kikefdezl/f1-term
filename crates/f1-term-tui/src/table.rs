use f1_term_core::driver::Driver;
use f1_term_core::team::Team;
use f1_term_core::timing::LiveTiming;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row, Table as RatatuiTable, Widget};

pub struct TableData {
    line: Option<u8>,
    driver_tla: String,
    driver_number: String,
    team_color: Color,
    last_lap_time: Option<String>,
}

pub struct TableDataArgs<'a> {
    pub driver: &'a Driver,
    pub team: &'a Team,
    pub live_timing: Option<&'a LiveTiming>,
}

impl TableData {
    pub fn from(args: &TableDataArgs) -> Self {
        TableData {
            line: args.driver.line,
            driver_tla: args.driver.tla.clone(),
            driver_number: args.driver.number.value.to_string(),
            team_color: Color::from_u32(args.team.color.u32),
            last_lap_time: args.live_timing.map(|lt| lt.last_lap.time.clone()),
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

        let mut rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, data)| {
                let pos = i + 1;

                let last_lap = match &data.last_lap_time {
                    Some(ll) => ll.clone(),
                    None => "-:--:---".to_string(),
                };

                Row::new(vec![
                    Cell::from(format!("{:>3}", pos)),
                    Cell::from(data.driver_tla.clone()).style(
                        Style::default()
                            .fg(data.team_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(data.driver_number.clone()),
                    Cell::from(last_lap),
                ])
            })
            .collect();

        rows.insert(
            0,
            Row::new(vec![
                Cell::from("···").style(Style::default().fg(Color::DarkGray)),
                Cell::from("······").style(Style::default().fg(Color::DarkGray)),
                Cell::from("···").style(Style::default().fg(Color::DarkGray)),
                Cell::from("······").style(Style::default().fg(Color::DarkGray)),
            ]),
        );

        let header = Row::new(vec![
            Cell::from(Line::from("#").alignment(ratatui::layout::Alignment::Right)),
            Cell::from("Driver"),
            Cell::from("Num"),
            Cell::from("Last Lap"),
        ]);

        let t = RatatuiTable::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Min(3),
                Constraint::Min(6),
            ],
        )
        .header(header);
        t.render(area, buf);
    }
}
