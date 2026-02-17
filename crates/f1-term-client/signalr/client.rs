use super::parsing::parse_message;
use super::topic::Topic;
use f1_term_core::client::{F1Client, TelemetryEvent};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

const URL: &str = "livetiming.formula1.com/signalr";
const HUB: &str = "Streaming";

type TcpWebSocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct SignalRF1Client {
    reader: Option<SplitStream<TcpWebSocketStream>>,
    writer: Option<SplitSink<TcpWebSocketStream, Message>>,
    topics: Vec<Topic>,
}

impl Default for SignalRF1Client {
    fn default() -> SignalRF1Client {
        SignalRF1Client {
            reader: None,
            writer: None,
            topics: Topic::all(),
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

impl F1Client for SignalRF1Client {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("=== Negotiating connection ===");

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
        println!("Connection token: {}", negotiate_data.connection_token);
        println!("Cookie: {}", cookie);

        let ws_url = Url::parse_with_params(
            &format!("wss://{}/connect", URL),
            &[
                ("clientProtocol", "1.5"),
                ("transport", "webSockets"),
                ("connectionToken", &negotiate_data.connection_token),
                ("connectionData", &connection_data),
            ],
        )?;

        println!("\n=== Connecting to WebSocket ===");
        println!("URL: {}", ws_url);

        let mut req = ws_url.to_string().into_client_request()?;
        let headers = req.headers_mut();
        headers.insert("Cookie", cookie.parse().unwrap());

        let (ws_stream, _) = connect_async(req).await?;
        println!("✓ WebSocket connected!");

        let (writer, reader) = ws_stream.split();
        self.writer = Some(writer);
        self.reader = Some(reader);

        println!("\n === Subscribing to topics ===");
        self.subscribe().await?;
        Ok(())
    }

    async fn next_event(&mut self) -> Option<TelemetryEvent> {
        let reader = self.reader.as_mut()?;

        loop {
            match reader.next().await? {
                Ok(Message::Text(text)) => {
                    if let Some(event) = parse_message(&text) {
                        return Some(event);
                    }
                    // Continue looping if we got a message we don't care about
                }
                Ok(Message::Close(_)) => {
                    println!("Connection closed");
                    return None;
                }
                Ok(Message::Ping(_) | Message::Pong(_)) => {
                    // Ignore ping/pong, continue loop
                }
                Ok(Message::Binary(_)) => {
                    // Skip binary messages for now
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    return None;
                }
                _ => {}
            }
        }
    }
}
