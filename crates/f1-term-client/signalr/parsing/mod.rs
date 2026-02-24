use std::{collections::HashMap, sync::Arc};

use f1_term_core::{
    client::TelemetryEvent,
    driver::{Driver, DriverNumber},
    session::Session,
    team::{Team, TeamName},
};
use log::{debug, error, info};

use self::{
    drivers::parse_drivers, stints::parse_stints, teams::parse_teams,
    timing_data::parse_timing_data,
};
use super::topic::Topic;
use crate::signalr::parsing::session_info::parse_session_info;

mod drivers;
mod session_info;
mod stints;
mod teams;
mod timing_data;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_message(state: &serde_json::Value) -> Option<TelemetryEvent> {
    if let Some(obj) = state.as_object() {
        let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        debug!("Canonical state contains topics: {:?}", keys);
    } else {
        debug!("Canonical state is not an object: {:?}", state);
    }

    let (drivers, teams) = match state.get(Topic::DriverList.to_string()) {
        None => (HashMap::new(), HashMap::new()),
        Some(dl) => {
            // TODO: If either of these fail right now the whole thing fails, but
            // this shouldn't be and we will need incremental updates
            let drivers: HashMap<DriverNumber, Driver> = parse_drivers(dl).ok()?;
            let teams: HashMap<TeamName, Team> = parse_teams(dl).ok()?;
            (drivers, teams)
        }
    };

    let info = match state.get(Topic::SessionInfo.to_string()) {
        None => {
            error!("No message from the SessionInfo topic!");
            return None;
        }
        Some(si) => parse_session_info(si).ok()?,
    };

    let timing_data = match state.get(Topic::TimingData.to_string()) {
        None => HashMap::new(),
        Some(td) => parse_timing_data(td).unwrap_or_else(|e| {
            info!("Failed to parse timing data: {}", e);
            HashMap::new()
        }),
    };

    let stints = match state.get(Topic::TimingAppData.to_string()) {
        None => HashMap::new(),
        Some(td) => parse_stints(td).unwrap_or_else(|e| {
            info!("Failed to parse stints: {}", e);
            HashMap::new()
        }),
    };

    let snapshot = Session {
        info,
        drivers,
        teams,
        timing_data,
        stints,
    };
    Some(TelemetryEvent::SessionUpdate(Arc::new(snapshot)))
}
