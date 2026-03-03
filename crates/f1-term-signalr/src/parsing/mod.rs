use std::collections::HashMap;

use f1_term_core::{
    driver::{Driver, DriverNumber},
    team::{Team, TeamName},
    telemetry_provider::TelemetryUpdate,
};
use log::info;

use self::{
    driver_list::{parse_drivers, parse_teams},
    stints::parse_stints,
    timing_data::parse_timing_data,
};
use super::{
    parsing::{
        race_control_messages::parse_race_control_messages, session_info::parse_session_info,
        track_status::parse_track_status, weather_data::parse_weather_data,
    },
    topic::Topic,
};

pub mod driver_list;
pub mod race_control_messages;
pub mod session_info;
pub mod stints;
pub mod timing_data;
pub mod track_status;
pub mod weather_data;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_message(state: &serde_json::Value, updated_topics: &[String]) -> Vec<TelemetryUpdate> {
    let mut events = Vec::new();

    for topic_str in updated_topics {
        if let Some(topic_data) = state.get(topic_str) {
            if topic_str == &Topic::DriverList.to_string() {
                let drivers: HashMap<DriverNumber, Driver> =
                    parse_drivers(topic_data).unwrap_or_default();
                let teams: HashMap<TeamName, Team> = parse_teams(topic_data).unwrap_or_default();
                events.push(TelemetryUpdate::DriverList(drivers, teams));
            } else if topic_str == &Topic::SessionInfo.to_string() {
                if let Ok(info) = parse_session_info(topic_data) {
                    events.push(TelemetryUpdate::SessionInfo(Box::new(info)));
                }
            } else if topic_str == &Topic::TimingData.to_string() {
                if let Ok(timing_data) = parse_timing_data(topic_data) {
                    events.push(TelemetryUpdate::TimingData(timing_data));
                } else {
                    info!("Failed to parse timing data");
                }
            } else if topic_str == &Topic::TimingAppData.to_string() {
                if let Ok(stints) = parse_stints(topic_data) {
                    events.push(TelemetryUpdate::Stints(stints));
                } else {
                    info!("Failed to parse stints");
                }
            } else if topic_str == &Topic::TrackStatus.to_string() {
                if let Ok(track_status) = parse_track_status(topic_data) {
                    events.push(TelemetryUpdate::TrackStatus(track_status));
                }
            } else if topic_str == &Topic::RaceControlMessages.to_string() {
                if let Ok(race_control_messages) = parse_race_control_messages(topic_data) {
                    events.push(TelemetryUpdate::RaceControlMessages(race_control_messages));
                } else {
                    info!("Failed to parse race control messages");
                }
            } else if topic_str == &Topic::WeatherData.to_string()
                && let Ok(weather) = parse_weather_data(topic_data)
            {
                events.push(TelemetryUpdate::Weather(weather));
            }
        }
    }

    events
}
