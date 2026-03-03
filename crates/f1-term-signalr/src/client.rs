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

use super::{parsing::parse_message, topic::Topic};

const URL: &str = "livetiming.formula1.com/signalr";
const HUB: &str = "Streaming";

type TcpWebSocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct SignalRF1Client {
    reader: Option<SplitStream<TcpWebSocketStream>>,
    writer: Option<SplitSink<TcpWebSocketStream, Message>>,
    topics: Vec<Topic>,
    canonical_state: serde_json::Value,
}

impl Default for SignalRF1Client {
    fn default() -> SignalRF1Client {
        SignalRF1Client {
            reader: None,
            writer: None,
            topics: Topic::all(),
            canonical_state: json!({}),
        }
    }
}

impl SignalRF1Client {
    pub fn new() -> SignalRF1Client {
        SignalRF1Client::default()
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
        let reader = self.reader.as_mut()?;

        loop {
            match reader.next().await? {
                Ok(Message::Text(text)) => {
                    debug!("Received SignalR Text Message. Length: {}", text.len());
                    let mut updated_topics = Vec::new();

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        // 1. Initial State (the 'R' field from Subscribe)
                        if let Some(r) = json.get("R") {
                            debug!("Applying initial state (R field)");
                            self.canonical_state.clone_from(r);
                            if let Some(obj) = self.canonical_state.as_object() {
                                updated_topics.extend(obj.keys().cloned());
                            }
                        }

                        // 2. Partial Updates (the 'M' array of messages)
                        if let Some(m_arr) = json.get("M").and_then(|m| m.as_array()) {
                            for msg in m_arr {
                                if msg.get("M").and_then(|m| m.as_str()) == Some("feed")
                                    && let Some(args) = msg.get("A").and_then(|a| a.as_array())
                                    && args.len() >= 2
                                {
                                    let topic = args[0].as_str().unwrap_or("UnknownTopic");
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
                                    if !updated_topics.contains(&topic.to_string()) {
                                        updated_topics.push(topic.to_string());
                                    }
                                }
                            }
                        }
                    }

                    if !updated_topics.is_empty() {
                        let updates = parse_message(&self.canonical_state, &updated_topics);
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

// Deep JSON merge patch (RFC 7386)
fn merge_patch(a: &mut serde_json::Value, b: &serde_json::Value) {
    use serde_json::Value;

    match (a, b) {
        (Value::Object(a_obj), Value::Object(b_obj)) => {
            for (k, v) in b_obj {
                if v.is_null() {
                    a_obj.remove(k);
                } else if let Some(target) = a_obj.get_mut(k) {
                    merge_patch(target, v);
                } else {
                    a_obj.insert(k.clone(), v.clone());
                }
            }
        }
        (a_val, b_val) => {
            a_val.clone_from(b_val);
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_merge_patch_basic() {
        let mut a = json!({
            "existing": "old",
            "nested": { "child": "value", "child2": "value2"},
            "array": [1, 2],
            "to_remove": "bye"
        });

        let b = json!({
            "existing": "new",
            "new_key": 42,
            "nested": { "child": "new", "child3": "value3" },
            "array": [3, 4],
            "to_remove": null
        });

        merge_patch(&mut a, &b);

        assert_eq!(
            a,
            json!({
                "existing": "new",
                "new_key": 42,
                "nested": { "child": "new", "child2": "value2" , "child3": "value3"},
                "array": [3, 4]
            })
        );
    }
}
