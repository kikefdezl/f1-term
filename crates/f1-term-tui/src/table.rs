use f1_term_core::driver::Driver;
use f1_term_core::team::Team;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row, Table as RatatuiTable, Widget};

pub struct TableData {
    line: Option<u8>,
    driver_tla: String,
    driver_number: String,
    team_color: Color,
}

impl TableData {
    pub fn from_driver_team(driver: &Driver, team: &Team) -> Self {
        TableData {
            line: driver.line,
            driver_tla: driver.tla.clone(),
            driver_number: driver.number.value.to_string(),
            team_color: Color::from_u32(team.color.u32),
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
                Row::new(vec![
                    Cell::from(format!("{:>3}", pos)),
                    Cell::from(data.driver_tla.clone()).style(
                        Style::default()
                            .fg(data.team_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(data.driver_number.clone()),
                ])
            })
            .collect();

        rows.insert(
            0,
            Row::new(vec![
                Cell::from("···").style(Style::default().fg(Color::DarkGray)),
                Cell::from("······").style(Style::default().fg(Color::DarkGray)),
                Cell::from("···").style(Style::default().fg(Color::DarkGray)),
            ]),
        );

        let header = Row::new(vec![
            Cell::from(Line::from("#").alignment(ratatui::layout::Alignment::Right)),
            Cell::from("Driver"),
            Cell::from("Num"),
        ]);

        let t = RatatuiTable::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Min(3),
            ],
        )
        .header(header);
        t.render(area, buf);
    }
}
