use std::collections::HashMap;
use std::time::Duration;

use super::clock::Clock;
use super::driver::{Driver, DriverNumber};
use super::laps::Laps;
use super::session_info::QualiPhase;
use super::stint::Stints;
use super::team::{Team, TeamName};
use super::timing::LiveTiming;
use super::track_status::TrackStatus;
use super::weather::Weather;
use crate::circuit::Circuit;
use crate::gap::Gap;
use crate::race_control_message::RaceControlMessage;
use crate::session_info::{SessionInfo, SessionType};
use crate::telemetry_provider::TelemetryUpdate;

#[derive(Debug, Default, Clone)]
pub struct TelemetryState {
    pub update_version: u64,
    pub delay: Duration,
    pub info: Option<SessionInfo>,
    pub drivers: HashMap<DriverNumber, Driver>,
    pub teams: HashMap<TeamName, Team>,
    pub timing_data: HashMap<DriverNumber, LiveTiming>,
    pub stints: HashMap<DriverNumber, Stints>,
    pub track_status: Option<TrackStatus>,
    pub race_control_messages: Vec<RaceControlMessage>,
    pub circuit: Option<Circuit>,
    pub weather: Option<Weather>,
    pub laps: Option<Laps>,
    pub clock: Option<Clock>,
}

#[derive(Debug)]
pub struct ParticipantContext<'a> {
    pub driver: &'a Driver,
    pub team: &'a Team,
    pub timing: Option<&'a LiveTiming>,
    pub stints: Option<&'a Stints>,
}

impl<'a> ParticipantContext<'a> {
    pub fn time_diff_to_fastest(&self, session_type: Option<&SessionType>) -> Option<Gap> {
        self.timing.and_then(|lt| {
            if let Some(SessionType::Qualifying(Some(phase))) = session_type {
                return lt
                    .quali_stats
                    .as_ref()
                    .and_then(|qs| match phase {
                        QualiPhase::Q1 => qs.q1_diffs.as_ref(),
                        QualiPhase::Q2 => qs.q2_diffs.as_ref(),
                        QualiPhase::Q3 => qs.q3_diffs.as_ref(),
                    })
                    .and_then(|d| d.to_fastest);
            }
            lt.time_diffs.to_fastest
        })
    }

    pub fn time_diff_to_position_ahead(&self, session_type: Option<&SessionType>) -> Option<Gap> {
        self.timing.and_then(|lt| {
            if let Some(SessionType::Qualifying(Some(phase))) = session_type {
                return lt
                    .quali_stats
                    .as_ref()
                    .and_then(|qs| match phase {
                        QualiPhase::Q1 => qs.q1_diffs.as_ref(),
                        QualiPhase::Q2 => qs.q2_diffs.as_ref(),
                        QualiPhase::Q3 => qs.q3_diffs.as_ref(),
                    })
                    .and_then(|d| d.to_position_ahead);
            }
            lt.time_diffs.to_position_ahead
        })
    }
}

impl TelemetryState {
    pub fn participants(&self) -> Vec<ParticipantContext<'_>> {
        let mut contexts = Vec::with_capacity(self.drivers.len());

        for driver in self.drivers.values() {
            let team = self
                .teams
                .get(&driver.team_name)
                .expect("Driver must have an associated team");

            contexts.push(ParticipantContext {
                driver,
                team,
                timing: self.timing_data.get(&driver.number),
                stints: self.stints.get(&driver.number),
            });
        }

        // Sort by current race position (timing.position) then grid position (driver.line)
        contexts.sort_by(|a, b| {
            let pos_a = a.timing.map(|t| t.position).unwrap_or(u8::MAX);
            let pos_b = b.timing.map(|t| t.position).unwrap_or(u8::MAX);

            if pos_a != pos_b {
                pos_a.cmp(&pos_b)
            } else {
                let line_a = a.driver.line.unwrap_or(u8::MAX);
                let line_b = b.driver.line.unwrap_or(u8::MAX);
                line_a.cmp(&line_b)
            }
        });

        contexts
    }

    pub fn apply(&mut self, update: TelemetryUpdate) -> bool {
        let mut changed = false;

        if let Some(info) = update.session_info {
            self.info = Some(*info);
            changed = true;
        }

        if let Some(mut circuit) = update.circuit {
            if let Some(old_circuit) = self.circuit.take()
                && old_circuit.key == circuit.key
            {
                circuit.layout = old_circuit.layout;
            }
            self.circuit = Some(circuit);
            changed = true;
        }

        if let Some(layout) = update.circuit_layout
            && let Some(circuit) = &mut self.circuit
        {
            circuit.layout = Some(layout);
            changed = true;
        }

        if let Some(drivers) = update.drivers {
            self.drivers.extend(drivers);
            changed = true;
        }

        if let Some(teams) = update.teams {
            self.teams.extend(teams);
            changed = true;
        }

        if let Some(timing_data) = update.timing_data {
            self.timing_data.extend(timing_data);
            changed = true;
        }

        if let Some(stints) = update.stints {
            self.stints.extend(stints);
            changed = true;
        }

        if let Some(track_status) = update.track_status {
            self.track_status = Some(track_status);
            changed = true;
        }

        if let Some(messages) = update.race_control_messages {
            self.race_control_messages = messages;
            changed = true;
        }

        if let Some(weather) = update.weather {
            self.weather = Some(weather);
            changed = true;
        }

        if let Some(laps) = update.laps {
            self.laps = Some(laps);
            changed = true;
        }

        if let Some(clock) = update.clock {
            self.clock = Some(clock);
            changed = true;
        }

        changed
    }
}

#[cfg(test)]
mod time_diff_tests {
    use super::*;
    use crate::driver::{Driver, DriverNumber};
    use crate::lap_time::LapTime;
    use crate::session_info::{QualiPhase, SessionType};
    use crate::team::{Team, TeamColor, TeamName};
    use crate::timing::{BestLap, LapData, LastLap, LiveTiming, PitData, QualiStats, TimeDiffs};

    fn create_context(timing: Option<LiveTiming>) -> ParticipantContext<'static> {
        let driver = Box::new(Driver {
            number: DriverNumber { value: 1 },
            broadcast_name: "VER".to_string(),
            full_name: "Max Verstappen".to_string(),
            first_name: "Max".to_string(),
            last_name: "Verstappen".to_string(),
            team_name: TeamName {
                value: "Red Bull Racing".to_string(),
            },
            line: Some(1),
            headshot_url: "".to_string(),
            public_id_right: "".to_string(),
            tla: "VER".to_string(),
            reference: "".to_string(),
        });

        let team = Box::new(Team {
            name: TeamName {
                value: "Red Bull Racing".to_string(),
            },
            color: TeamColor { u32: 0x1e41ff },
        });

        ParticipantContext {
            driver: Box::leak(driver),
            team: Box::leak(team),
            timing: timing.map(|t| Box::leak(Box::new(t)) as &'static LiveTiming),
            stints: None,
        }
    }

    fn create_timing(regular_diffs: TimeDiffs, quali_stats: Option<QualiStats>) -> LiveTiming {
        LiveTiming {
            driver_number: DriverNumber { value: 1 },
            position: 1,
            lap_data: LapData {
                best_lap: BestLap {
                    time: None,
                    overall_fastest: false,
                },
                last_lap: LastLap::default(),
                number_of_laps: None,
            },
            pit_data: PitData::default(),
            time_diffs: regular_diffs,
            quali_stats,
            stopped: false,
            retired: false,
            status: 0,
        }
    }

    fn create_quali_stats() -> QualiStats {
        QualiStats {
            cutoff: None,
            knocked_out: None,
            q1_diffs: Some(TimeDiffs {
                to_fastest: Some(Gap::Time(LapTime::from_millis(100))),
                to_position_ahead: Some(Gap::Time(LapTime::from_millis(50))),
            }),
            q2_diffs: Some(TimeDiffs {
                to_fastest: Some(Gap::Time(LapTime::from_millis(200))),
                to_position_ahead: Some(Gap::Time(LapTime::from_millis(150))),
            }),
            q3_diffs: None,
        }
    }

    fn create_time_diffs() -> TimeDiffs {
        TimeDiffs {
            to_fastest: Some(Gap::Time(LapTime::from_seconds(1))),
            to_position_ahead: Some(Gap::Time(LapTime::from_millis(500))),
        }
    }

    #[test]
    fn test_time_diff_to_fastest_no_timing() {
        let ctx = create_context(None);
        assert_eq!(ctx.time_diff_to_fastest(None), None);
    }

    #[test]
    fn test_time_diff_to_fastest_regular_session() {
        let timing = create_timing(create_time_diffs(), None);
        let ctx = create_context(Some(timing));
        assert_eq!(
            ctx.time_diff_to_fastest(Some(&SessionType::Practice)),
            Some(Gap::Time(LapTime::from_seconds(1)))
        );
        assert_eq!(
            ctx.time_diff_to_fastest(Some(&SessionType::Race)),
            Some(Gap::Time(LapTime::from_seconds(1)))
        );
        assert_eq!(
            ctx.time_diff_to_fastest(None),
            Some(Gap::Time(LapTime::from_seconds(1)))
        );
    }

    #[test]
    fn test_time_diff_to_fastest_quali_no_stats() {
        let timing = create_timing(create_time_diffs(), None);
        let ctx = create_context(Some(timing));
        assert_eq!(
            ctx.time_diff_to_fastest(Some(&SessionType::Qualifying(Some(QualiPhase::Q1)))),
            None
        );
    }

    #[test]
    fn test_time_diff_to_fastest_quali_with_stats() {
        let timing = create_timing(create_time_diffs(), Some(create_quali_stats()));
        let ctx = create_context(Some(timing));

        assert_eq!(
            ctx.time_diff_to_fastest(Some(&SessionType::Qualifying(Some(QualiPhase::Q1)))),
            Some(Gap::Time(LapTime::from_millis(100)))
        );
        assert_eq!(
            ctx.time_diff_to_fastest(Some(&SessionType::Qualifying(Some(QualiPhase::Q2)))),
            Some(Gap::Time(LapTime::from_millis(200)))
        );
        assert_eq!(
            ctx.time_diff_to_fastest(Some(&SessionType::Qualifying(Some(QualiPhase::Q3)))),
            None
        );
        assert_eq!(
            ctx.time_diff_to_fastest(Some(&SessionType::Qualifying(None))),
            Some(Gap::Time(LapTime::from_seconds(1)))
        );
    }

    #[test]
    fn test_time_diff_to_position_ahead_no_timing() {
        let ctx = create_context(None);
        assert_eq!(ctx.time_diff_to_position_ahead(None), None);
    }

    #[test]
    fn test_time_diff_to_position_ahead_regular_session() {
        let timing = create_timing(create_time_diffs(), None);
        let ctx = create_context(Some(timing));
        assert_eq!(
            ctx.time_diff_to_position_ahead(Some(&SessionType::Practice)),
            Some(Gap::Time(LapTime::from_millis(500)))
        );
        assert_eq!(
            ctx.time_diff_to_position_ahead(Some(&SessionType::Race)),
            Some(Gap::Time(LapTime::from_millis(500)))
        );
        assert_eq!(
            ctx.time_diff_to_position_ahead(None),
            Some(Gap::Time(LapTime::from_millis(500)))
        );
    }

    #[test]
    fn test_time_diff_to_position_ahead_quali_with_stats() {
        let timing = create_timing(create_time_diffs(), Some(create_quali_stats()));
        let ctx = create_context(Some(timing));

        assert_eq!(
            ctx.time_diff_to_position_ahead(Some(&SessionType::Qualifying(Some(QualiPhase::Q1)))),
            Some(Gap::Time(LapTime::from_millis(50)))
        );
        assert_eq!(
            ctx.time_diff_to_position_ahead(Some(&SessionType::Qualifying(Some(QualiPhase::Q2)))),
            Some(Gap::Time(LapTime::from_millis(150)))
        );
        assert_eq!(
            ctx.time_diff_to_position_ahead(Some(&SessionType::Qualifying(Some(QualiPhase::Q3)))),
            None
        );
        assert_eq!(
            ctx.time_diff_to_position_ahead(Some(&SessionType::Qualifying(None))),
            Some(Gap::Time(LapTime::from_millis(500)))
        );
    }
}
