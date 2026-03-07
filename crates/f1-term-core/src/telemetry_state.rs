use std::collections::HashMap;

use super::{
    driver::{Driver, DriverNumber},
    laps::Laps,
    stint::Stints,
    team::{Team, TeamName},
    timing::LiveTiming,
    track_status::TrackStatus,
    weather::Weather,
};
use crate::{
    race_control_message::RaceControlMessage, session_info::SessionInfo,
    telemetry_provider::TelemetryUpdate,
};

#[derive(Debug, Default, Clone)]
pub struct TelemetryState {
    pub update_version: u64,
    pub info: Option<SessionInfo>,
    pub teams: HashMap<TeamName, Team>,
    pub drivers: HashMap<DriverNumber, Driver>,
    pub timing_data: HashMap<DriverNumber, LiveTiming>,
    pub stints: HashMap<DriverNumber, Stints>,
    pub track_status: Option<TrackStatus>,
    pub race_control_messages: Vec<RaceControlMessage>,
    pub weather: Option<Weather>,
    pub laps: Option<Laps>,
}

#[derive(Debug, Clone, Copy)]
pub enum SessionType {
    Practice,
    Qualifying { current_phase: usize },
    Race,
}

#[derive(Debug)]
pub struct ParticipantContext<'a> {
    pub driver: &'a Driver,
    pub team: &'a Team,
    pub timing: Option<&'a LiveTiming>,
    pub stints: Option<&'a Stints>,
    pub session_type: Option<SessionType>,
}

impl<'a> ParticipantContext<'a> {
    pub fn time_diff_to_fastest(&self) -> Option<String> {
        self.timing.and_then(|lt| {
            if let Some(SessionType::Qualifying { current_phase }) = self.session_type
                && current_phase > 0
            {
                return lt
                    .quali_stats
                    .as_ref()
                    .and_then(|qs| qs.diffs.as_ref())
                    .and_then(|stats| {
                        if stats.len() == current_phase {
                            stats.last().and_then(|s| s.to_fastest.clone())
                        } else {
                            None
                        }
                    });
            }
            lt.time_diffs.to_fastest.clone()
        })
    }

    pub fn time_diff_to_position_ahead(&self) -> Option<String> {
        self.timing.and_then(|lt| {
            if let Some(SessionType::Qualifying { current_phase }) = self.session_type
                && current_phase > 0
            {
                return lt
                    .quali_stats
                    .as_ref()
                    .and_then(|qs| qs.diffs.as_ref())
                    .and_then(|stats| {
                        if stats.len() == current_phase {
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

        let session_type = self.info.as_ref().map(|info| match info.type_ {
            crate::session_info::SessionType::Practice => SessionType::Practice,
            crate::session_info::SessionType::Qualifying => {
                let current_phase = self
                    .timing_data
                    .values()
                    .filter_map(|timing| {
                        timing.quali_stats.as_ref().and_then(|qs| qs.diffs.as_ref())
                    })
                    .map(|stats| stats.len())
                    .max()
                    .unwrap_or(0);
                SessionType::Qualifying { current_phase }
            }
            crate::session_info::SessionType::Race => SessionType::Race,
        });

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
                session_type,
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
        match update {
            TelemetryUpdate::SessionInfo(mut info) => {
                if let Some(mut old_info) = self.info.take()
                    && old_info.meeting.circuit.key == info.meeting.circuit.key
                {
                    info.meeting.circuit.layout = old_info.meeting.circuit.layout.take();
                }
                self.info = Some(*info);
                true
            }
            TelemetryUpdate::DriverList(drivers, teams) => {
                self.drivers = drivers;
                self.teams = teams;
                true
            }
            TelemetryUpdate::TimingData(timing_data) => {
                self.timing_data = timing_data;
                true
            }
            TelemetryUpdate::Stints(stints) => {
                self.stints = stints;
                true
            }
            TelemetryUpdate::TrackStatus(track_status) => {
                self.track_status = Some(track_status);
                true
            }
            TelemetryUpdate::RaceControlMessages(messages) => {
                self.race_control_messages = messages;
                true
            }
            TelemetryUpdate::Weather(weather) => {
                self.weather = Some(weather);
                true
            }
            TelemetryUpdate::Laps(laps) => {
                self.laps = Some(laps);
                true
            }
            TelemetryUpdate::Empty => false,
        }
    }
}
