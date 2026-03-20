use std::fmt::Display;

use crossterm::event::KeyCode;
use f1_term_core::driver::Driver;
use f1_term_core::lap_time::LapTime;
use f1_term_core::stint::{Compound, Stints};
use f1_term_core::team::Team;
use f1_term_core::telemetry_state::TelemetryState;
use f1_term_core::timing::{LiveTiming, Sector, Segment, SegmentStatus};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table as RatatuiTable, TableState};

use super::{Action, Component};
use crate::constants::{
    COLOR_ABORTED, COLOR_HARD, COLOR_IN_PIT, COLOR_INTERMEDIATE, COLOR_MEDIUM,
    COLOR_OVERALL_FASTEST, COLOR_PERSONAL_FASTEST, COLOR_SLOWER, COLOR_SOFT, COLOR_UNKNOWN,
    COLOR_WET, SEGMENT_WIDTH, SEGMENTS,
};

#[derive(Default)]
pub struct TimingTableData {
    driver_tla: String,
    driver_number: String,
    team_color: Color,
    tire_compound: Option<Compound>,
    tire_laps: Option<u8>,
    retired: bool,
    stopped: bool,
    in_pit: bool,
    best_lap_time: Option<LapTime>,
    best_lap_overall_fastest: bool,
    last_lap_time: Option<LapTime>,
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
    pub time_diff_to_fastest: Option<String>,
    pub time_diff_to_position_ahead: Option<String>,
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

        self.retired = args.live_timing.map(|lt| lt.retired).unwrap_or(false);
        self.stopped = args.live_timing.map(|lt| lt.stopped).unwrap_or(false);
        self.in_pit = args
            .live_timing
            .map(|lt| lt.pit_data.in_pit)
            .unwrap_or(false);

        self.best_lap_time.clone_from(
            &args
                .live_timing
                .and_then(|lt| lt.lap_data.best_lap.time.clone()),
        );
        self.best_lap_overall_fastest = args
            .live_timing
            .map(|lt| lt.lap_data.best_lap.overall_fastest)
            .unwrap_or(false);

        self.last_lap_time.clone_from(
            &args
                .live_timing
                .and_then(|lt| lt.lap_data.last_lap.time.clone()),
        );
        self.last_lap_overall_fastest = args
            .live_timing
            .map(|lt| lt.lap_data.last_lap.overall_fastest)
            .unwrap_or(false);
        self.last_lap_personal_fastest = args
            .live_timing
            .map(|lt| lt.lap_data.last_lap.personal_fastest)
            .unwrap_or(false);

        if let Some(lt) = args.live_timing {
            self.sectors.clone_from(&lt.lap_data.last_lap.sectors);
        } else {
            self.sectors.clear();
        }

        self.time_diff_to_fastest = args.time_diff_to_fastest.clone();
        self.time_diff_to_position_ahead = args.time_diff_to_position_ahead.clone();
    }

    fn position_cell(&self, pos: usize) -> Cell<'_> {
        let pos_color = if self.retired || self.stopped {
            Color::DarkGray
        } else {
            Color::default()
        };
        Cell::from(format!("{:>3}", pos)).style(Style::default().fg(pos_color))
    }

    fn driver_tla_cell(&self) -> Cell<'_> {
        Cell::from(self.driver_tla.clone()).style(
            Style::default()
                .fg(self.team_color)
                .add_modifier(Modifier::BOLD),
        )
    }

    fn number_cell(&self) -> Cell<'_> {
        Cell::from(self.driver_number.as_str())
    }

    fn tire_cell(&self) -> Cell<'_> {
        match (&self.tire_compound, self.tire_laps) {
            (Some(compound), Some(laps)) => {
                let (letter, color) = match compound {
                    Compound::Soft => ("S", COLOR_SOFT),
                    Compound::Medium => ("M", COLOR_MEDIUM),
                    Compound::Hard => ("H", COLOR_HARD),
                    Compound::Wet => ("W", COLOR_WET),
                    Compound::Intermediate => ("I", COLOR_INTERMEDIATE),
                    Compound::Unknown => ("?", COLOR_UNKNOWN),
                };
                Cell::from(format!("{} ({})", letter, laps)).style(Style::default().fg(color))
            }
            _ => Cell::from(""),
        }
    }

    fn pit_cell(&self) -> Cell<'_> {
        if self.retired || self.stopped {
            Cell::from("Out").style(Style::default().fg(Color::DarkGray))
        } else {
            match self.in_pit {
                true => Cell::from("Pit").style(Style::default().fg(Color::Blue)),
                false => Cell::from(""),
            }
        }
    }

    fn best_lap_cell(&self) -> Cell<'_> {
        let (best_lap, color) = match &self.best_lap_time {
            Some(ll) => {
                let color = match self.best_lap_overall_fastest {
                    true => COLOR_OVERALL_FASTEST,
                    false => COLOR_PERSONAL_FASTEST,
                };
                (ll.to_string(), color)
            }
            None => ("-:--.---".to_string(), Color::default()),
        };

        Cell::from(best_lap).style(Style::default().fg(color))
    }

    fn last_lap_cell(&self) -> Cell<'_> {
        let (last_lap, color) = match &self.last_lap_time {
            Some(ll) => {
                let color = if self.last_lap_overall_fastest {
                    COLOR_OVERALL_FASTEST
                } else if self.last_lap_personal_fastest {
                    COLOR_PERSONAL_FASTEST
                } else {
                    COLOR_SLOWER
                };
                (ll.to_string(), color)
            }
            None => ("-:--.---".to_string(), Color::default()),
        };

        Cell::from(last_lap).style(Style::default().fg(color))
    }

    fn gap_cell(&self, is_leader: bool, gap_mode: GapMode) -> Cell<'_> {
        if is_leader {
            Cell::from("------")
        } else {
            let diff = match gap_mode {
                GapMode::ToFastest => &self.time_diff_to_fastest,
                GapMode::ToPositionAhead => &self.time_diff_to_position_ahead,
            };
            match diff {
                Some(t) => Cell::from(t.as_str()),
                None => Cell::from(" -.---"),
            }
        }
    }

    fn sector_cell(&self, sector: usize) -> Cell<'_> {
        self.sectors
            .get(sector)
            .map(|s| TimingTable::sector(s))
            .unwrap_or(Cell::from(""))
    }

    fn segment_cell(&self, sector: usize) -> Cell<'_> {
        self.sectors
            .get(sector)
            .map(|s| TimingTable::segments(&s.segments))
            .unwrap_or_default()
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

impl Display for GapMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GapMode::ToFastest => write!(f, "Gap")?,
            GapMode::ToPositionAhead => write!(f, "Int")?,
        };
        Ok(())
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
        let participants = state.participants();
        let session_type = state.info.as_ref().map(|info| &info.type_);

        if self.items.len() < participants.len() {
            self.items
                .resize_with(participants.len(), TimingTableData::default);
        } else {
            self.items.truncate(participants.len());
        }

        for (i, participant) in participants.into_iter().enumerate() {
            let args = TimingTableDataArgs {
                driver: participant.driver,
                team: participant.team,
                live_timing: participant.timing,
                stints: participant.stints,
                time_diff_to_fastest: participant.time_diff_to_fastest(session_type),
                time_diff_to_position_ahead: participant.time_diff_to_position_ahead(session_type),
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
        let header = self.header();

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
    fn header(&self) -> Row<'static> {
        Row::new(vec![
            Cell::from("  #"),
            Cell::from("Drv"),
            Cell::from("Num"),
            Cell::from("Tire"),
            Cell::from("Sts"),
            Cell::from("Best Lap"),
            Cell::from(self.gap_mode.to_string()),
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
                let is_leader = i == 0;

                Row::new(vec![
                    data.position_cell(pos),
                    data.driver_tla_cell(),
                    data.number_cell(),
                    data.tire_cell(),
                    data.pit_cell(),
                    data.best_lap_cell(),
                    data.gap_cell(is_leader, gap_mode),
                    data.last_lap_cell(),
                    data.segment_cell(0),
                    data.sector_cell(0),
                    data.segment_cell(1),
                    data.sector_cell(1),
                    data.segment_cell(2),
                    data.sector_cell(2),
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
                    SegmentStatus::InPit => COLOR_IN_PIT,
                    SegmentStatus::OverallFastest => COLOR_OVERALL_FASTEST,
                    SegmentStatus::PersonalFastest => COLOR_PERSONAL_FASTEST,
                    SegmentStatus::Aborted => COLOR_ABORTED,
                    SegmentStatus::Normal => COLOR_SLOWER,
                    SegmentStatus::Unknown => Color::White,
                };
                Span::styled(SEGMENTS, Style::default().fg(color))
            })
            .collect();

        Cell::from(Line::from(spans))
    }
}
#[cfg(test)]
mod tests {
    use f1_term_core::stint::Compound;

    use super::*;

    #[test]
    fn test_gap_mode_toggle() {
        let mut mode = GapMode::ToFastest;
        assert_eq!(mode.to_string(), "Gap");

        mode.toggle();
        assert_eq!(mode, GapMode::ToPositionAhead);
        assert_eq!(mode.to_string(), "Int");

        mode.toggle();
        assert_eq!(mode, GapMode::ToFastest);
    }

    #[test]
    fn test_pit_cell() {
        let mut data = TimingTableData::default();

        assert_eq!(data.pit_cell(), Cell::from(""));

        data.in_pit = true;
        assert_eq!(
            data.pit_cell(),
            Cell::from("Pit").style(Style::default().fg(Color::Blue))
        );

        data.retired = true;
        assert_eq!(
            data.pit_cell(),
            Cell::from("Out").style(Style::default().fg(Color::DarkGray))
        );
    }

    #[test]
    fn test_tire_cell() {
        let mut data = TimingTableData::default();
        assert_eq!(data.tire_cell(), Cell::from(""));

        data.tire_compound = Some(Compound::Soft);
        data.tire_laps = Some(5);
        assert_eq!(
            data.tire_cell(),
            Cell::from("S (5)").style(Style::default().fg(Color::Red))
        );

        data.tire_compound = Some(Compound::Medium);
        data.tire_laps = Some(12);
        assert_eq!(
            data.tire_cell(),
            Cell::from("M (12)").style(Style::default().fg(Color::Yellow))
        );
    }

    #[test]
    fn test_gap_cell() {
        let mut data = TimingTableData {
            time_diff_to_fastest: Some("+1.234".to_string()),
            time_diff_to_position_ahead: Some("+0.500".to_string()),
            ..Default::default()
        };

        assert_eq!(
            data.gap_cell(true, GapMode::ToFastest),
            Cell::from("------")
        );
        assert_eq!(
            data.gap_cell(true, GapMode::ToPositionAhead),
            Cell::from("------")
        );

        assert_eq!(
            data.gap_cell(false, GapMode::ToFastest),
            Cell::from("+1.234")
        );

        assert_eq!(
            data.gap_cell(false, GapMode::ToPositionAhead),
            Cell::from("+0.500")
        );

        data.time_diff_to_fastest = None;
        assert_eq!(
            data.gap_cell(false, GapMode::ToFastest),
            Cell::from(" -.---")
        );
    }

    #[test]
    fn test_lap_time_cells() {
        let mut data = TimingTableData::default();

        // No lap times
        assert_eq!(
            data.best_lap_cell(),
            Cell::from("-:--.---").style(Style::default().fg(Color::default()))
        );
        assert_eq!(
            data.last_lap_cell(),
            Cell::from("-:--.---").style(Style::default().fg(Color::default()))
        );

        // With lap times
        data.best_lap_time = Some(LapTime::new(1, 20, 0));
        data.last_lap_time = Some(LapTime::new(1, 21, 0));

        assert_eq!(
            data.best_lap_cell(),
            Cell::from("1:20.000").style(Style::default().fg(COLOR_PERSONAL_FASTEST))
        );

        // Last lap is default color
        assert_eq!(
            data.last_lap_cell(),
            Cell::from("1:21.000").style(Style::default().fg(COLOR_SLOWER))
        );

        // Last lap is personal best
        data.last_lap_personal_fastest = true;
        assert_eq!(
            data.last_lap_cell(),
            Cell::from("1:21.000").style(Style::default().fg(COLOR_PERSONAL_FASTEST))
        );

        // Last lap is overall best (should override personal best)
        data.last_lap_overall_fastest = true;
        assert_eq!(
            data.last_lap_cell(),
            Cell::from("1:21.000").style(Style::default().fg(COLOR_OVERALL_FASTEST))
        );
    }
}
