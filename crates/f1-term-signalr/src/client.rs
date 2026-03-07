use std::{
    fs::{self, OpenOptions},
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use f1_term_core::telemetry_provider::{TelemetryProvider, TelemetryUpdate};
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

use super::{merge_patch::merge_patch, topic::Topic};

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

    async fn next_updates(&mut self) -> Option<Vec<TelemetryUpdate>> {
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
                        let updates = self.extract_updates(&updated_topics);
                        info!("Parsed {} delta updates", updates.len());
                        return Some(updates);
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

    fn extract_updates(&self, updated_topics: &[Topic]) -> Vec<TelemetryUpdate> {
        let mut events = Vec::new();

        self.extract_session_update(updated_topics, &mut events);

        for topic in updated_topics {
            if matches!(topic, &Topic::SessionInfo | &Topic::SessionData) {
                continue;
            }

            if let Some(topic_data) = self.canonical_state.get(topic.to_string()) {
                self.extract_topic_update(topic, topic_data, &mut events);
            }
        }

        events
    }

    fn extract_session_update(&self, updated_topics: &[Topic], events: &mut Vec<TelemetryUpdate>) {
        let session_info_updated = updated_topics.contains(&Topic::SessionInfo);
        let session_data_updated = updated_topics.contains(&Topic::SessionData);

        if !(session_info_updated || session_data_updated) {
            return;
        }

        let Some(info_data) = self.canonical_state.get(Topic::SessionInfo.to_string()) else {
            return;
        };

        let session_data = self.canonical_state.get(Topic::SessionData.to_string());
        match crate::parsing::session_info::parse_raw_session_info(info_data) {
            Ok(raw_info) => {
                let raw_data =
                    session_data.and_then(crate::parsing::session_data::parse_raw_session_data);
                match crate::convert::session::convert_session_info(&raw_info, raw_data.as_ref()) {
                    Ok(info) => events.push(TelemetryUpdate::SessionInfo(Box::new(info))),
                    Err(e) => error!("{}", e),
                }
            }
            Err(e) => error!("{}", e),
        }
    }

    fn extract_topic_update(
        &self,
        topic: &Topic,
        topic_data: &serde_json::Value,
        events: &mut Vec<TelemetryUpdate>,
    ) {
        match topic {
            Topic::DriverList => match crate::parsing::driver_list::parse_driver_list(topic_data) {
                Ok(raw_drivers) => {
                    let drivers = crate::convert::driver::convert_drivers(&raw_drivers);
                    events.push(TelemetryUpdate::Drivers(drivers));
                    let teams = crate::convert::team::convert_teams(&raw_drivers);
                    events.push(TelemetryUpdate::Teams(teams));
                }
                Err(e) => error!("{}", e),
            },
            Topic::TimingData => {
                match crate::parsing::timing_data::parse_raw_timing_data(topic_data) {
                    Ok(raw_timing) => events.push(TelemetryUpdate::TimingData(
                        crate::convert::timing::convert_timing_data(&raw_timing),
                    )),
                    Err(e) => error!("{}", e),
                }
            }
            Topic::TimingAppData => match crate::parsing::stints::parse_raw_stints(topic_data) {
                Ok(raw_stints) => {
                    let stints = crate::convert::stint::convert_stints(&raw_stints);
                    events.push(TelemetryUpdate::Stints(stints));
                }
                Err(e) => error!("{}", e),
            },
            Topic::TrackStatus => {
                match crate::parsing::track_status::parse_raw_track_status(topic_data) {
                    Ok(raw_status) => {
                        match crate::convert::track_status::convert_track_status(&raw_status) {
                            Ok(track_status) => {
                                events.push(TelemetryUpdate::TrackStatus(track_status))
                            }
                            Err(e) => error!("{}", e),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Topic::RaceControlMessages => {
                match crate::parsing::race_control_messages::parse_raw_race_control_messages(
                    topic_data,
                ) {
                    Ok(raw_messages) => {
                        match crate::convert::race_control_message::convert_race_control_messages(
                            &raw_messages.Messages,
                        ) {
                            Ok(race_control_messages) => events
                                .push(TelemetryUpdate::RaceControlMessages(race_control_messages)),
                            Err(e) => error!("{}", e),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Topic::WeatherData => {
                match crate::parsing::weather_data::parse_raw_weather_data(topic_data) {
                    Ok(raw_weather) => {
                        match crate::convert::weather::convert_weather_data(&raw_weather) {
                            Ok(weather) => events.push(TelemetryUpdate::Weather(weather)),
                            Err(e) => error!("{}", e),
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            Topic::LapCount => match crate::parsing::lap_count::parse_raw_lap_count(topic_data) {
                Ok(raw_laps) => events.push(TelemetryUpdate::Laps(
                    crate::convert::lap_count::convert_lap_count(&raw_laps),
                )),
                Err(e) => error!("{}", e),
            },
            _ => {}
        }
    }
}
