use std::{collections::HashMap, future::Future};

use crate::{
    circuit::{Circuit, CircuitLayout},
    driver::{Driver, DriverNumber},
    laps::Laps,
    race_control_message::RaceControlMessage,
    session_info::SessionInfo,
    stint::Stints,
    team::{Team, TeamName},
    timing::LiveTiming,
    track_status::TrackStatus,
    weather::Weather,
};

#[derive(Debug, Default)]
pub struct TelemetryUpdate {
    pub session_info: Option<Box<SessionInfo>>,
    pub circuit: Option<Circuit>,
    pub circuit_layout: Option<CircuitLayout>,
    pub drivers: Option<HashMap<DriverNumber, Driver>>,
    pub teams: Option<HashMap<TeamName, Team>>,
    pub timing_data: Option<HashMap<DriverNumber, LiveTiming>>,
    pub stints: Option<HashMap<DriverNumber, Stints>>,
    pub track_status: Option<TrackStatus>,
    pub race_control_messages: Option<Vec<RaceControlMessage>>,
    pub weather: Option<Weather>,
    pub laps: Option<Laps>,
}

pub trait TelemetryProvider {
    fn connect(&mut self) -> impl Future<Output = Result<(), Box<dyn std::error::Error>>>;
    fn next_updates(&mut self) -> impl Future<Output = Option<TelemetryUpdate>>;
}
