use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use f1_term_core::{
    driver::{Driver, DriverNumber},
    race_control_message::RaceControlMessage,
    session_info::SessionInfo,
    stint::Stints,
    team::{Team, TeamName},
    telemetry_provider::{TelemetryProvider, TelemetryUpdate},
    timing::LiveTiming,
    track_status::TrackStatus,
    weather::Weather,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use log::{debug, error, info, warn};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, client::IntoClientRequest},
};

use crate::{
    convert::{
        driver::convert_drivers, lap_count::convert_lap_count,
        race_control_message::convert_race_control_messages, session::convert_session_info,
        stint::convert_stints, team::convert_teams, timing::convert_timing_data,
        track_status::convert_track_status, weather::convert_weather_data,
    },
    merge_patch::merge_patch,
    parsing::{
        driver_list::parse_driver_list, lap_count::parse_raw_lap_count,
        race_control_messages::parse_raw_race_control_messages,
        session_data::parse_raw_session_data, session_info::parse_raw_session_info,
        stints::parse_raw_stints, timing_data::parse_raw_timing_data,
        track_status::parse_raw_track_status, weather_data::parse_raw_weather_data,
    },
    topic::Topic,
};

const URL: &str = "livetiming.formula1.com/signalr";
const HUB: &str = "Streaming";

type TcpWebSocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct SignalRF1Client {
    reader: Option<SplitStream<TcpWebSocketStream>>,
    writer: Option<SplitSink<TcpWebSocketStream, Message>>,
    topics: Vec<Topic>,
    canonical_state: serde_json::Value,
    log_dir: Option<String>,
}

impl Default for SignalRF1Client {
    fn default() -> SignalRF1Client {
        SignalRF1Client {
            reader: None,
            writer: None,
            topics: Topic::all(),
            canonical_state: json!({}),
            log_dir: None,
        }
    }
}

impl SignalRF1Client {
    pub fn new() -> SignalRF1Client {
        SignalRF1Client::default()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct NegotiateResponse {
    connection_token: String,
}

impl TelemetryProvider for SignalRF1Client {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("=== Negotiating connection ===");

        let connection_data = json!([{"name": HUB}]).to_string();
        let negotiate_url = format!("https://{}/negotiate", URL);

        let client = reqwest::Client::new();
        let negotiate_response = client.get(&negotiate_url).send().await?;

        let cookie = negotiate_response
            .headers()
            .get("set-cookie")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        let negotiate_data: NegotiateResponse = negotiate_response.json().await?;
        debug!("Connection token: {}", negotiate_data.connection_token);
        debug!("Cookie: {}", cookie);

        let ws_url = Url::parse_with_params(
            &format!("wss://{}/connect", URL),
            &[
                ("clientProtocol", "1.5"),
                ("transport", "webSockets"),
                ("connectionToken", &negotiate_data.connection_token),
                ("connectionData", &connection_data),
            ],
        )?;

        info!("=== Connecting to WebSocket ===");
        info!("URL: {}", ws_url);

        let mut req = ws_url.to_string().into_client_request()?;
        let headers = req.headers_mut();
        headers.insert("Cookie", cookie.parse().unwrap());

        let (ws_stream, _) = connect_async(req).await?;
        info!("✓ WebSocket connected!");

        let (writer, reader) = ws_stream.split();
        self.writer = Some(writer);
        self.reader = Some(reader);

        info!("\n === Subscribing to topics ===");
        self.subscribe().await?;
        Ok(())
    }

    async fn next_updates(&mut self) -> Option<TelemetryUpdate> {
        loop {
            let msg = {
                let reader = self.reader.as_mut()?;
                reader.next().await?
            };

            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received SignalR Text Message. Length: {}", text.len());
                    let mut updated_topics: Vec<Topic> = Vec::new();

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        // 1. Initial State (the 'R' field from Subscribe)
                        if let Some(r) = json.get("R") {
                            debug!("Applying initial state (R field)");
                            self.canonical_state.clone_from(r);

                            if let Some(obj) = self.canonical_state.as_object() {
                                updated_topics.extend(
                                    obj.keys().filter_map(|k| Topic::try_from(k.as_str()).ok()),
                                );
                                for (k, v) in obj {
                                    self.log_topic_update(k, v);
                                }
                            }
                        }

                        // 2. Partial Updates (the 'M' array of messages)
                        if let Some(m_arr) = json.get("M").and_then(|m| m.as_array()) {
                            for msg in m_arr {
                                if msg.get("M").and_then(|m| m.as_str()) == Some("feed")
                                    && let Some(args) = msg.get("A").and_then(|a| a.as_array())
                                    && args.len() >= 2
                                {
                                    let Ok(topic) =
                                        Topic::try_from(args[0].as_str().unwrap_or("UnknownTopic"))
                                    else {
                                        warn!("Unknown topic {}", args[0]);
                                        continue;
                                    };

                                    let delta = &args[1];
                                    debug!("Applying partial update for topic: {}", topic);

                                    if !self.canonical_state.is_object() {
                                        self.canonical_state = json!({});
                                    }

                                    let canonical_obj =
                                        self.canonical_state.as_object_mut().unwrap();
                                    let topic_entry = canonical_obj
                                        .entry(topic.to_string())
                                        .or_insert_with(|| json!({}));
                                    merge_patch(topic_entry, delta);

                                    self.log_topic_update(&topic.to_string(), delta);

                                    if !updated_topics.contains(&topic) {
                                        updated_topics.push(topic);
                                    }
                                }
                            }
                        }
                    }

                    if !updated_topics.is_empty() {
                        return Some(self.extract_updates(&updated_topics));
                    }
                }
                Ok(Message::Close(_)) => {
                    warn!("Connection closed by server");
                    return None;
                }
                Ok(Message::Ping(_) | Message::Pong(_)) => {
                    debug!("Received Ping/Pong");
                }
                Ok(Message::Binary(_)) => {
                    debug!("Received unhandled Binary message");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    return None;
                }
                _ => {}
            }
        }
    }
}

impl SignalRF1Client {
    pub fn with_log_dir(mut self, log_dir: String) -> Self {
        self.log_dir = Some(log_dir);
        self
    }

    fn log_topic_update(&self, topic: &str, payload: &serde_json::Value) {
        let base_log_dir = match &self.log_dir {
            Some(dir) => dir,
            None => return,
        };

        // (e.g. "2024/2024-03-02_Bahrain_Grand_Prix/...")
        let session_path = self
            .canonical_state
            .get("SessionInfo")
            .and_then(|info| info.get("Path"))
            .and_then(|path| path.as_str())
            .unwrap_or("unknown_session");

        let final_log_dir = format!(
            "{}/{}",
            base_log_dir.trim_end_matches('/'),
            session_path.trim_start_matches('/')
        );

        if let Err(e) = fs::create_dir_all(&final_log_dir) {
            error!("Failed to create log directory {}: {}", final_log_dir, e);
            return;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let filename = format!("{}/{}.log", final_log_dir.trim_end_matches('/'), topic);

        let mut file = match OpenOptions::new().create(true).append(true).open(&filename) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open log file {}: {}", filename, e);
                return;
            }
        };

        let payload_str = match serde_json::to_string(payload) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to serialize payload for {}: {}", topic, e);
                return;
            }
        };

        if let Err(e) = writeln!(file, "[{}]: {}", timestamp, payload_str) {
            error!("Failed to write to {}: {}", filename, e);
        }
    }

    pub async fn subscribe(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &mut self.writer {
            None => return Err("Client not connected".into()),
            Some(w) => {
                let subscribe_msg = json!({
                    "H": HUB,
                    "M": "Subscribe",
                    "A": [self.topics.iter().map(|t| t.to_string()).collect::<Vec<String>>()],
                    "I": 1
                });
                w.send(Message::Text(subscribe_msg.to_string().into()))
                    .await?
            }
        }
        Ok(())
    }

    fn extract_updates(&self, updated_topics: &[Topic]) -> TelemetryUpdate {
        TelemetryUpdate {
            session_info: self.extract_session_info_update(updated_topics),
            drivers: self.extract_drivers_update(updated_topics),
            teams: self.extract_teams_update(updated_topics),
            timing_data: self.extract_timing_data_update(updated_topics),
            stints: self.extract_stints_update(updated_topics),
            track_status: self.extract_track_status_update(updated_topics),
            race_control_messages: self.extract_race_control_messages_update(updated_topics),
            weather: self.extract_weather_update(updated_topics),
            laps: self.extract_lap_count_update(updated_topics),
        }
    }

    fn extract_session_info_update(&self, updated_topics: &[Topic]) -> Option<Box<SessionInfo>> {
        if !(updated_topics.contains(&Topic::SessionInfo)
            || updated_topics.contains(&Topic::SessionData))
        {
            return None;
        }

        let info_data = self.canonical_state.get(Topic::SessionInfo.to_string())?;

        match parse_raw_session_info(info_data) {
            Ok(raw_info) => {
                let session_data = self.canonical_state.get(Topic::SessionData.to_string());
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

    fn extract_drivers_update(
        &self,
        updated_topics: &[Topic],
    ) -> Option<HashMap<DriverNumber, Driver>> {
        if !updated_topics.contains(&Topic::DriverList) {
            return None;
        }

        let topic_data = self.canonical_state.get(Topic::DriverList.to_string())?;

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

    fn extract_teams_update(&self, updated_topics: &[Topic]) -> Option<HashMap<TeamName, Team>> {
        if !updated_topics.contains(&Topic::DriverList) {
            return None;
        }

        let topic_data = self.canonical_state.get(Topic::DriverList.to_string())?;

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
        &self,
        updated_topics: &[Topic],
    ) -> Option<HashMap<DriverNumber, LiveTiming>> {
        if !updated_topics.contains(&Topic::TimingData) {
            return None;
        }

        let topic_data = self.canonical_state.get(Topic::TimingData.to_string())?;

        match parse_raw_timing_data(topic_data) {
            Ok(raw_timing) => Some(convert_timing_data(&raw_timing)),
            Err(e) => {
                error!("{}", e);
                None
            }
        }
    }

    fn extract_stints_update(
        &self,
        updated_topics: &[Topic],
    ) -> Option<HashMap<DriverNumber, Stints>> {
        if !updated_topics.contains(&Topic::TimingAppData) {
            return None;
        }

        let topic_data = self.canonical_state.get(Topic::TimingAppData.to_string())?;

        match parse_raw_stints(topic_data) {
            Ok(raw_stints) => Some(convert_stints(&raw_stints)),
            Err(e) => {
                error!("{}", e);
                None
            }
        }
    }

    fn extract_track_status_update(&self, updated_topics: &[Topic]) -> Option<TrackStatus> {
        if !updated_topics.contains(&Topic::TrackStatus) {
            return None;
        }

        let topic_data = self.canonical_state.get(Topic::TrackStatus.to_string())?;

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
        &self,
        updated_topics: &[Topic],
    ) -> Option<Vec<RaceControlMessage>> {
        if !updated_topics.contains(&Topic::RaceControlMessages) {
            return None;
        }

        let topic_data = self
            .canonical_state
            .get(Topic::RaceControlMessages.to_string())?;

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

    fn extract_weather_update(&self, updated_topics: &[Topic]) -> Option<Weather> {
        if !updated_topics.contains(&Topic::WeatherData) {
            return None;
        }

        let topic_data = self.canonical_state.get(Topic::WeatherData.to_string())?;

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
        &self,
        updated_topics: &[Topic],
    ) -> Option<f1_term_core::laps::Laps> {
        if !updated_topics.contains(&Topic::LapCount) {
            return None;
        }

        let topic_data = self.canonical_state.get(Topic::LapCount.to_string())?;

        match parse_raw_lap_count(topic_data) {
            Ok(raw_laps) => Some(convert_lap_count(&raw_laps)),
            Err(e) => {
                error!("{}", e);
                None
            }
        }
    }
}
