use std::collections::HashMap;

use f1_term_core::{
    driver::{Driver, DriverNumber},
    team::{Team, TeamName},
    telemetry_provider::TelemetryUpdate,
};
use log::error;

use self::{
    driver_list::{parse_drivers, parse_teams},
    stints::parse_stints,
    timing_data::parse_timing_data,
};
use super::{
    parsing::{
        lap_count::parse_lap_count, race_control_messages::parse_race_control_messages,
        session_info::parse_session_info, track_status::parse_track_status,
        weather_data::parse_weather_data,
    },
    topic::Topic,
};

pub mod driver_list;
pub mod lap_count;
pub mod race_control_messages;
pub mod session_data;
pub mod session_info;
pub mod stints;
pub mod timing_data;
pub mod track_status;
pub mod weather_data;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_message(state: &serde_json::Value, updated_topics: &[Topic]) -> Vec<TelemetryUpdate> {
    let mut events = Vec::new();

    let session_info_updated = updated_topics.contains(&Topic::SessionInfo);
    let session_data_updated = updated_topics.contains(&Topic::SessionData);

    if (session_info_updated || session_data_updated)
        && let Some(info_data) = state.get(Topic::SessionInfo.to_string())
    {
        let session_data = state.get(Topic::SessionData.to_string());
        match parse_session_info(info_data, session_data) {
            Ok(info) => events.push(TelemetryUpdate::SessionInfo(Box::new(info))),
            Err(e) => error!("{}", e),
        }
    }

    for topic in updated_topics {
        if matches!(topic, &Topic::SessionInfo | &Topic::SessionData) {
            continue;
        }

        if let Some(topic_data) = state.get(topic.to_string()) {
            if topic == &Topic::DriverList {
                let drivers: HashMap<DriverNumber, Driver> =
                    parse_drivers(topic_data).unwrap_or_default();
                events.push(TelemetryUpdate::Drivers(drivers));
                let teams: HashMap<TeamName, Team> = parse_teams(topic_data).unwrap_or_default();
                events.push(TelemetryUpdate::Teams(teams));
            } else if topic == &Topic::TimingData {
                match parse_timing_data(topic_data) {
                    Ok(timing_data) => events.push(TelemetryUpdate::TimingData(timing_data)),
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::TimingAppData {
                match parse_stints(topic_data) {
                    Ok(stints) => events.push(TelemetryUpdate::Stints(stints)),
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::TrackStatus {
                match parse_track_status(topic_data) {
                    Ok(track_status) => events.push(TelemetryUpdate::TrackStatus(track_status)),
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::RaceControlMessages {
                match parse_race_control_messages(topic_data) {
                    Ok(race_control_messages) => {
                        events.push(TelemetryUpdate::RaceControlMessages(race_control_messages))
                    }
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::WeatherData {
                match parse_weather_data(topic_data) {
                    Ok(weather) => events.push(TelemetryUpdate::Weather(weather)),
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::LapCount {
                match parse_lap_count(topic_data) {
                    Ok(laps) => events.push(TelemetryUpdate::Laps(laps)),
                    Err(e) => error!("{}", e),
                }
            }
        }
    }

    events
}
