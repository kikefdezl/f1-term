use std::{collections::HashMap, future::Future};

use crate::{
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

#[derive(Debug)]
pub enum TelemetryUpdate {
    SessionInfo(Box<SessionInfo>),
    Drivers(HashMap<DriverNumber, Driver>),
    Teams(HashMap<TeamName, Team>),
    TimingData(HashMap<DriverNumber, LiveTiming>),
    Stints(HashMap<DriverNumber, Stints>),
    TrackStatus(TrackStatus),
    RaceControlMessages(Vec<RaceControlMessage>),
    Weather(Weather),
    Laps(Laps),
    Empty,
}

pub trait TelemetryProvider {
    fn connect(&mut self) -> impl Future<Output = Result<(), Box<dyn std::error::Error>>>;
    fn next_updates(&mut self) -> impl Future<Output = Option<Vec<TelemetryUpdate>>>;
}
