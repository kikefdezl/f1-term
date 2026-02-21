use self::drivers::parse_drivers;
use self::stints::parse_stints;
use self::teams::parse_teams;
use self::timing_data::parse_timing_data;
use super::topic::Topic;
use f1_term_core::client::TelemetryEvent;
use f1_term_core::driver::{Driver, DriverNumber};
use f1_term_core::snapshot::FullSnapshot;
use f1_term_core::team::{Team, TeamName};
use log::info;
use std::collections::HashMap;

mod drivers;
mod stints;
mod teams;
mod timing_data;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_message(text: &str) -> Option<TelemetryEvent> {
    let json: serde_json::Value = serde_json::from_str(text).ok()?;

    let response = json.get("R")?;

    let (drivers, teams) = match response.get(Topic::DriverList.to_string()) {
        None => (HashMap::new(), HashMap::new()),
        Some(dl) => {
            // TODO: If either of these fail right now the whole thing fails, but
            // this shouldn't be and we will need incremental updates
            let drivers: HashMap<DriverNumber, Driver> = parse_drivers(dl).ok()?;
            let teams: HashMap<TeamName, Team> = parse_teams(dl).ok()?;
            (drivers, teams)
        }
    };

    let timing_data = match response.get(Topic::TimingData.to_string()) {
        None => HashMap::new(),
        Some(td) => parse_timing_data(td).unwrap_or_else(|e| {
            info!("Failed to parse timing data: {}", e);
            HashMap::new()
        }),
    };

    let stints = match response.get(Topic::TimingAppData.to_string()) {
        None => HashMap::new(),
        Some(td) => parse_stints(td).unwrap_or_else(|e| {
            info!("Failed to parse stints: {}", e);
            HashMap::new()
        }),
    };

    let snapshot = FullSnapshot {
        drivers,
        teams,
        timing_data,
        stints,
    };
    Some(TelemetryEvent::Full(snapshot))
}
