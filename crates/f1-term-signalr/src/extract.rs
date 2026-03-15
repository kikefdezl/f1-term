use std::collections::HashMap;

use f1_term_core::circuit::Circuit;
use f1_term_core::clock::Clock;
use f1_term_core::driver::{Driver, DriverNumber};
use f1_term_core::laps::Laps;
use f1_term_core::race_control_message::RaceControlMessage;
use f1_term_core::session_info::SessionInfo;
use f1_term_core::stint::Stints;
use f1_term_core::team::{Team, TeamName};
use f1_term_core::telemetry_provider::TelemetryUpdate;
use f1_term_core::timing::LiveTiming;
use f1_term_core::track_status::TrackStatus;
use f1_term_core::weather::Weather;
use log::error;

use crate::convert::circuit::convert_circuit;
use crate::convert::clock::convert_clock;
use crate::convert::driver::convert_drivers;
use crate::convert::lap_count::convert_lap_count;
use crate::convert::race_control_message::convert_race_control_messages;
use crate::convert::session::convert_session_info;
use crate::convert::stint::convert_stints;
use crate::convert::team::convert_teams;
use crate::convert::timing::convert_timing_data;
use crate::convert::track_status::convert_track_status;
use crate::convert::weather::convert_weather_data;
use crate::parsing::driver_list::parse_driver_list;
use crate::parsing::extrapolated_clock::parse_extrapolated_clock;
use crate::parsing::lap_count::parse_raw_lap_count;
use crate::parsing::race_control_messages::parse_raw_race_control_messages;
use crate::parsing::session_data::parse_raw_session_data;
use crate::parsing::session_info::parse_raw_session_info;
use crate::parsing::stints::parse_raw_stints;
use crate::parsing::timing_data::parse_raw_timing_data;
use crate::parsing::track_status::parse_raw_track_status;
use crate::parsing::weather_data::parse_raw_weather_data;
use crate::topic::Topic;

pub fn extract_updates(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> TelemetryUpdate {
    TelemetryUpdate {
        session_info: extract_session_info_update(canonical_state, updated_topics),
        circuit: extract_circuit_update(canonical_state, updated_topics),
        circuit_layout: None,
        drivers: extract_drivers_update(canonical_state, updated_topics),
        teams: extract_teams_update(canonical_state, updated_topics),
        timing_data: extract_timing_data_update(canonical_state, updated_topics),
        stints: extract_stints_update(canonical_state, updated_topics),
        track_status: extract_track_status_update(canonical_state, updated_topics),
        race_control_messages: extract_race_control_messages_update(
            canonical_state,
            updated_topics,
        ),
        weather: extract_weather_update(canonical_state, updated_topics),
        laps: extract_lap_count_update(canonical_state, updated_topics),
        clock: extract_clock_update(canonical_state, updated_topics),
    }
}

fn extract_session_info_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<Box<SessionInfo>> {
    if !(updated_topics.contains(&Topic::SessionInfo)
        || updated_topics.contains(&Topic::SessionData))
    {
        return None;
    }

    let info_data = canonical_state.get(Topic::SessionInfo.to_string())?;

    match parse_raw_session_info(info_data) {
        Ok(raw_info) => {
            let session_data = canonical_state.get(Topic::SessionData.to_string());
            let raw_data = session_data.and_then(parse_raw_session_data);
            match convert_session_info(&raw_info, raw_data.as_ref()) {
                Ok(info) => Some(Box::new(info)),
                Err(e) => {
                    error!("{}", e);
                    None
                }
            }
        }
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_circuit_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<Circuit> {
    if !(updated_topics.contains(&Topic::SessionInfo)
        || updated_topics.contains(&Topic::RaceControlMessages))
    {
        return None;
    }

    let info_data = canonical_state.get(Topic::SessionInfo.to_string())?;
    let rcm = canonical_state
        .get(Topic::RaceControlMessages.to_string())
        .and_then(|data| match parse_raw_race_control_messages(data) {
            Ok(messages) => Some(messages),
            Err(e) => {
                error!("Failed to parse race control messages for circuit: {}", e);
                None
            }
        });

    match parse_raw_session_info(info_data) {
        Ok(raw_info) => Some(convert_circuit(&raw_info, rcm.as_ref())),
        Err(e) => {
            error!("Failed to parse session info for circuit: {}", e);
            None
        }
    }
}

fn extract_drivers_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<HashMap<DriverNumber, Driver>> {
    if !updated_topics.contains(&Topic::DriverList) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::DriverList.to_string())?;

    match parse_driver_list(topic_data) {
        Ok(raw_drivers) => {
            let drivers = convert_drivers(&raw_drivers);
            Some(drivers)
        }
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_teams_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<HashMap<TeamName, Team>> {
    if !updated_topics.contains(&Topic::DriverList) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::DriverList.to_string())?;

    match parse_driver_list(topic_data) {
        Ok(raw_drivers) => {
            let teams = convert_teams(&raw_drivers);
            Some(teams)
        }
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_timing_data_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<HashMap<DriverNumber, LiveTiming>> {
    if !updated_topics.contains(&Topic::TimingData) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::TimingData.to_string())?;

    match parse_raw_timing_data(topic_data) {
        Ok(raw_timing) => Some(convert_timing_data(&raw_timing)),
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_stints_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<HashMap<DriverNumber, Stints>> {
    if !updated_topics.contains(&Topic::TimingAppData) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::TimingAppData.to_string())?;

    match parse_raw_stints(topic_data) {
        Ok(raw_stints) => Some(convert_stints(&raw_stints)),
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_track_status_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<TrackStatus> {
    if !updated_topics.contains(&Topic::TrackStatus) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::TrackStatus.to_string())?;

    match parse_raw_track_status(topic_data) {
        Ok(raw_status) => match convert_track_status(&raw_status) {
            Ok(track_status) => Some(track_status),
            Err(e) => {
                error!("{}", e);
                None
            }
        },
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_race_control_messages_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<Vec<RaceControlMessage>> {
    if !updated_topics.contains(&Topic::RaceControlMessages) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::RaceControlMessages.to_string())?;

    match parse_raw_race_control_messages(topic_data) {
        Ok(raw_messages) => match convert_race_control_messages(&raw_messages.Messages) {
            Ok(race_control_messages) => Some(race_control_messages),
            Err(e) => {
                error!("{}", e);
                None
            }
        },
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_weather_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<Weather> {
    if !updated_topics.contains(&Topic::WeatherData) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::WeatherData.to_string())?;

    match parse_raw_weather_data(topic_data) {
        Ok(raw_weather) => match convert_weather_data(&raw_weather) {
            Ok(weather) => Some(weather),
            Err(e) => {
                error!("{}", e);
                None
            }
        },
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_lap_count_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<Laps> {
    if !updated_topics.contains(&Topic::LapCount) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::LapCount.to_string())?;

    match parse_raw_lap_count(topic_data) {
        Ok(raw_laps) => Some(convert_lap_count(&raw_laps)),
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}

fn extract_clock_update(
    canonical_state: &serde_json::Value,
    updated_topics: &[Topic],
) -> Option<Clock> {
    if !updated_topics.contains(&Topic::ExtrapolatedClock) {
        return None;
    }

    let topic_data = canonical_state.get(Topic::ExtrapolatedClock.to_string())?;

    match parse_extrapolated_clock(topic_data) {
        Ok(raw_clock) => match convert_clock(&raw_clock) {
            Ok(c) => Some(c),
            Err(e) => {
                error!("{}", e);
                None
            }
        },
        Err(e) => {
            error!("{}", e);
            None
        }
    }
}
