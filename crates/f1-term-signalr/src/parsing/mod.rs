use std::collections::HashMap;

use f1_term_core::{
    driver::{Driver, DriverNumber},
    team::{Team, TeamName},
    telemetry_provider::TelemetryUpdate,
};
use log::error;

use self::{
    driver_list::parse_driver_list, lap_count::parse_raw_lap_count,
    session_info::parse_raw_session_info, stints::parse_raw_stints, timing_data::parse_raw_timing_data,
};
use super::{
    parsing::{
        race_control_messages::parse_raw_race_control_messages,
        track_status::parse_raw_track_status, weather_data::parse_raw_weather_data,
    },
    topic::Topic,
};
use crate::convert::{
    driver::convert_drivers, lap_count::convert_lap_count,
    race_control_message::convert_race_control_messages, session::convert_session_info,
    stint::convert_stints, team::convert_teams, track_status::convert_track_status,
    weather::convert_weather_data, timing::convert_timing_data,
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
        match parse_raw_session_info(info_data) {
            Ok(raw_info) => {
                let raw_data =
                    session_data.and_then(crate::parsing::session_data::parse_raw_session_data);
                match convert_session_info(&raw_info, raw_data.as_ref()) {
                    Ok(info) => events.push(TelemetryUpdate::SessionInfo(Box::new(info))),
                    Err(e) => error!("{}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
    }

    for topic in updated_topics {
        if matches!(topic, &Topic::SessionInfo | &Topic::SessionData) {
            continue;
        }

        if let Some(topic_data) = state.get(topic.to_string()) {
            if topic == &Topic::DriverList {
                match parse_driver_list(topic_data) {
                    Ok(raw_drivers) => {
                        let drivers: HashMap<DriverNumber, Driver> = convert_drivers(&raw_drivers);
                        events.push(TelemetryUpdate::Drivers(drivers));
                        let teams: HashMap<TeamName, Team> = convert_teams(&raw_drivers);
                        events.push(TelemetryUpdate::Teams(teams));
                    }
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::TimingData {
                match parse_raw_timing_data(topic_data) {
                    Ok(raw_timing) => events.push(TelemetryUpdate::TimingData(convert_timing_data(&raw_timing))),
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::TimingAppData {
                match parse_raw_stints(topic_data) {
                    Ok(raw_stints) => {
                        let stints = convert_stints(&raw_stints);
                        events.push(TelemetryUpdate::Stints(stints));
                    }
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::TrackStatus {
                match parse_raw_track_status(topic_data) {
                    Ok(raw_status) => match convert_track_status(&raw_status) {
                        Ok(track_status) => events.push(TelemetryUpdate::TrackStatus(track_status)),
                        Err(e) => error!("{}", e),
                    },
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::RaceControlMessages {
                match parse_raw_race_control_messages(topic_data) {
                    Ok(raw_messages) => {
                        match convert_race_control_messages(&raw_messages.Messages) {
                            Ok(race_control_messages) => events
                                .push(TelemetryUpdate::RaceControlMessages(race_control_messages)),
                            Err(e) => error!("{}", e),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::WeatherData {
                match parse_raw_weather_data(topic_data) {
                    Ok(raw_weather) => match convert_weather_data(&raw_weather) {
                        Ok(weather) => events.push(TelemetryUpdate::Weather(weather)),
                        Err(e) => error!("{}", e),
                    },
                    Err(e) => error!("{}", e),
                }
            } else if topic == &Topic::LapCount {
                match parse_raw_lap_count(topic_data) {
                    Ok(raw_laps) => {
                        events.push(TelemetryUpdate::Laps(convert_lap_count(&raw_laps)))
                    }
                    Err(e) => error!("{}", e),
                }
            }
        }
    }

    events
}
