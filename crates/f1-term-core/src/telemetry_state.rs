use std::collections::HashMap;

use super::{
    driver::{Driver, DriverNumber},
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
}

#[derive(Debug)]
pub struct ParticipantContext<'a> {
    pub driver: &'a Driver,
    pub team: &'a Team,
    pub timing: Option<&'a LiveTiming>,
    pub stints: Option<&'a Stints>,
}

impl TelemetryState {
    pub fn leaderboard(&self) -> Vec<ParticipantContext<'_>> {
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
            TelemetryUpdate::Empty => false,
        }
    }
}
