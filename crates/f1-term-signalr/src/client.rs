use std::error::Error;
use std::fs::{
    OpenOptions, {self},
};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use f1_term_core::telemetry_provider::{TelemetryProvider, TelemetryUpdate};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::extract::extract_updates;
use crate::merge_patch::merge_patch;
use crate::topic::Topic;

const URL: &str = "livetiming.formula1.com/signalrcore";
const RECORD_SEPARATOR: &str = "\u{001E}";

type TcpWebSocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct SignalRF1Client {
    reader: Option<SplitStream<TcpWebSocketStream>>,
    writer: Option<SplitSink<TcpWebSocketStream, Message>>,
    topics: Vec<Topic>,
    canonical_state: serde_json::Value,
    log_dir: Option<String>,
    base_url: String,
}

impl Default for SignalRF1Client {
    fn default() -> SignalRF1Client {
        SignalRF1Client {
            reader: None,
            writer: None,
            topics: Topic::all(),
            canonical_state: json!({}),
            log_dir: None,
            base_url: URL.to_string(),
        }
    }
}

impl SignalRF1Client {
    pub fn new() -> SignalRF1Client {
        SignalRF1Client::default()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NegotiateResponse {
    connection_token: Option<String>,
}

impl TelemetryProvider for SignalRF1Client {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("=== Negotiating connection ===");

        let client = reqwest::Client::builder().user_agent("BestHTTP").build()?;

        let (negotiate_data, cookie) = Self::negotiate(&client, &self.base_url).await?;

        let ws_scheme =
            if self.base_url.starts_with("localhost") || self.base_url.starts_with("127.0.0.1") {
                "ws"
            } else {
                "wss"
            };
        let mut ws_url = Url::parse(&format!("{}://{}", ws_scheme, self.base_url))?;
        if let Some(ref token) = negotiate_data.connection_token {
            ws_url.query_pairs_mut().append_pair("id", token);
        }

        let mut req = ws_url.to_string().into_client_request()?;
        let headers = req.headers_mut();

        headers.insert("User-Agent", "BestHTTP".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip,identity".parse().unwrap());

        if !cookie.is_empty() {
            headers.insert("Cookie", cookie.parse().unwrap());
        }

        let (mut ws_stream, _response) = connect_async(req).await?;

        Self::handshake(&mut ws_stream).await?;

        let (writer, reader) = ws_stream.split();
        self.writer = Some(writer);
        self.reader = Some(reader);

        self.subscribe().await?;
        info!("✓ Connected and subscribed to telemetry topics!");

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
                    let mut updated_topics: Vec<Topic> = Vec::new();

                    for msg_str in text.split(RECORD_SEPARATOR) {
                        let msg_str = msg_str.trim();
                        if msg_str.is_empty() {
                            continue;
                        }

                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(msg_str) {
                            self.process_signalr_message(json, &mut updated_topics);
                        }
                    }

                    if !updated_topics.is_empty() {
                        return Some(extract_updates(&self.canonical_state, &updated_topics));
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
                Ok(Message::Frame(_)) => {
                    debug!("Received unhandled Frame message");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    return None;
                }
            }
        }
    }
}

impl SignalRF1Client {
    pub fn with_log_dir(mut self, log_dir: String) -> Self {
        self.log_dir = Some(log_dir);
        self
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    async fn negotiate(
        client: &reqwest::Client,
        base_url: &str,
    ) -> Result<(NegotiateResponse, String), Box<dyn Error>> {
        let scheme = if base_url.starts_with("localhost") || base_url.starts_with("127.0.0.1") {
            "http"
        } else {
            "https"
        };
        let negotiate_url = format!("{}://{}/negotiate", scheme, base_url);

        debug!("Negotiating via OPTIONS to {}", negotiate_url);
        let options_res = client
            .request(reqwest::Method::OPTIONS, &negotiate_url)
            .send()
            .await?;

        let mut cookie = String::new();
        for header in options_res.headers().get_all("set-cookie") {
            if let Ok(s) = header.to_str()
                && let Some(cookie_part) = s.split(';').next()
            {
                let trimmed = cookie_part.trim();
                if trimmed.starts_with("AWSALBCORS=") {
                    cookie = trimmed.to_string();
                    break;
                }
            }
        }

        let url = Url::parse_with_params(&negotiate_url, &[("negotiateVersion", "1")])?;
        let mut req = client.post(url);
        if !cookie.is_empty() {
            req = req.header("Cookie", &cookie);
        }

        let res = req.send().await?;
        if !res.status().is_success() {
            return Err(format!("Negotiate failed with status: {}", res.status()).into());
        }

        let negotiate_data: NegotiateResponse = res.json().await?;
        Ok((negotiate_data, cookie))
    }

    async fn handshake(
        ws_stream: &mut TcpWebSocketStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let handshake_msg = format!(
            "{}{}",
            json!({"protocol": "json", "version": 1}),
            RECORD_SEPARATOR
        );
        ws_stream.send(Message::Text(handshake_msg.into())).await?;

        let handshake_response = ws_stream.next().await.ok_or_else(|| {
            Box::<dyn std::error::Error>::from("No handshake response received")
        })??;

        match handshake_response {
            Message::Text(txt) => {
                let stripped = txt.trim_end_matches(RECORD_SEPARATOR);
                let parsed: serde_json::Value = serde_json::from_str(stripped)?;
                if let Some(err) = parsed.get("error") {
                    return Err(format!("SignalR handshake error: {}", err).into());
                }
                debug!("Handshake successful!");
                Ok(())
            }
            other => Err(format!("Unexpected handshake response: {:?}", other).into()),
        }
    }

    pub async fn subscribe(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let writer = self.writer.as_mut().ok_or("Client not connected")?;

        let topics_str = self
            .topics
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>();

        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct InvocationMessage {
            #[serde(rename = "type")]
            r#type: i32,
            invocation_id: String,
            target: String,
            arguments: Vec<serde_json::Value>,
        }

        let invoke = InvocationMessage {
            r#type: 1, // INVOCATION
            invocation_id: "1".to_string(),
            target: "Subscribe".to_string(),
            arguments: vec![json!(topics_str)],
        };

        let msg_str = format!("{}{}", serde_json::to_string(&invoke)?, RECORD_SEPARATOR);
        writer.send(Message::Text(msg_str.into())).await?;
        Ok(())
    }

    fn process_signalr_message(
        &mut self,
        json: serde_json::Value,
        updated_topics: &mut Vec<Topic>,
    ) {
        let Some(msg_type) = json.get("type").and_then(|t| t.as_i64()) else {
            return;
        };

        match msg_type {
            3 => {
                // Completion message (contains the initial/canonical state in the 'result' field)
                if let Some(result) = json.get("result") {
                    self.canonical_state.clone_from(result);

                    if let Some(obj) = self.canonical_state.as_object() {
                        updated_topics
                            .extend(obj.keys().filter_map(|k| Topic::try_from(k.as_str()).ok()));
                        for (k, v) in obj {
                            self.log_topic_update(k, v);
                        }
                    }
                }
                if let Some(err) = json.get("error") {
                    error!("SignalR Core completion error: {}", err);
                }
            }
            1 => {
                // Invocation message (contains the live streaming feed)
                if json.get("target").and_then(|t| t.as_str()) == Some("feed")
                    && let Some(args) = json.get("arguments").and_then(|a| a.as_array())
                    && args.len() >= 2
                {
                    let topic_str = args[0].as_str().unwrap_or("UnknownTopic");
                    let Ok(topic) = Topic::try_from(topic_str) else {
                        warn!("Unknown topic {}", topic_str);
                        return;
                    };

                    let delta = &args[1];

                    if !self.canonical_state.is_object() {
                        self.canonical_state = json!({});
                    }

                    let canonical_obj = self.canonical_state.as_object_mut().unwrap();
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
            _ => {
                debug!("Received other SignalR Core message type: {}", msg_type);
            }
        }
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
}
