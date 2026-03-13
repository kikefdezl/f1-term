use std::collections::HashMap;

use super::{
    clock::Clock,
    driver::{Driver, DriverNumber},
    laps::Laps,
    stint::Stints,
    team::{Team, TeamName},
    timing::LiveTiming,
    track_status::TrackStatus,
    weather::Weather,
};
use crate::{
    circuit::Circuit,
    race_control_message::RaceControlMessage,
    session_info::{SessionInfo, SessionType},
    telemetry_provider::TelemetryUpdate,
};

#[derive(Debug, Default, Clone)]
pub struct TelemetryState {
    pub update_version: u64,
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
    pub fn time_diff_to_fastest(&self, session_type: Option<&SessionType>) -> Option<String> {
        self.timing.and_then(|lt| {
            if let Some(SessionType::Qualifying(Some(phase))) = session_type {
                return lt
                    .quali_stats
                    .as_ref()
                    .and_then(|qs| qs.diffs.as_ref())
                    .and_then(|stats| {
                        if stats.len() == phase.index() + 1 {
                            stats.last().and_then(|s| s.to_fastest.clone())
                        } else {
                            None
                        }
                    });
            }
            lt.time_diffs.to_fastest.clone()
        })
    }

    pub fn time_diff_to_position_ahead(
        &self,
        session_type: Option<&SessionType>,
    ) -> Option<String> {
        self.timing.and_then(|lt| {
            if let Some(SessionType::Qualifying(Some(phase))) = session_type {
                return lt
                    .quali_stats
                    .as_ref()
                    .and_then(|qs| qs.diffs.as_ref())
                    .and_then(|stats| {
                        if stats.len() == phase.index() + 1 {
                            stats.last().and_then(|s| s.to_position_ahead.clone())
                        } else {
                            None
                        }
                    });
            }
            lt.time_diffs.to_position_ahead.clone()
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
            self.drivers = drivers;
            changed = true;
        }

        if let Some(teams) = update.teams {
            self.teams = teams;
            changed = true;
        }

        if let Some(timing_data) = update.timing_data {
            self.timing_data = timing_data;
            changed = true;
        }

        if let Some(stints) = update.stints {
            self.stints = stints;
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
