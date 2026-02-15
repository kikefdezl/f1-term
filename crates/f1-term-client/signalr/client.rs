use std::collections::HashMap;

use super::topic::Topic;
use f1_term_core::team::{Team, TeamColor, TeamName};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use reqwest::Url;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use f1_term_core::client::{F1Client, TelemetryEvent};
use f1_term_core::driver::{Driver, DriverNumber};
use f1_term_core::snapshot::FullSnapshot;

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

fn parse_message(text: &str) -> Option<TelemetryEvent> {
    let json: serde_json::Value = serde_json::from_str(text).ok()?;

    let response = json.get("R")?;

    let (drivers, teams) = match response.get(Topic::DriverList.to_string()) {
        None => (HashMap::new(), HashMap::new()),
        Some(dl) => {
            let drivers: HashMap<DriverNumber, Driver> = parse_drivers(dl);
            let teams: HashMap<TeamName, Team> = parse_teams(dl);
            (drivers, teams)
        }
    };
    let snapshot = FullSnapshot { drivers, teams };
    Some(TelemetryEvent::Full(snapshot))
}

fn parse_drivers(val: &Value) -> HashMap<DriverNumber, Driver> {
    let mut drivers: HashMap<DriverNumber, Driver> = HashMap::new();
    match val {
        Value::Object(map) => {
            for (num, attrs) in map.iter() {
                let number: u8 = match num.parse() {
                    Ok(n) => n,
                    // Some non-grid cars have non-digits or numbers above 255. Ignore.
                    Err(_) => continue,
                };
                let driver_number = DriverNumber { value: number };
                // Medical and safety cars don't have all fields, so those fail to parse.
                // We just ignore them.
                if let Ok(d) = parse_driver(attrs) {
                    drivers.insert(driver_number, d);
                };
            }
        }
        // TODO: handle this properly
        _ => {}
    }
    drivers
}

fn parse_driver(val: &Value) -> Result<Driver, Box<dyn std::error::Error>> {
    match val {
        Value::Object(attrs) => Ok(Driver {
            number: DriverNumber {
                value: attrs
                    .get("RacingNumber")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid RacingNumber")?
                    .parse()?,
            },
            first_name: attrs
                .get("FirstName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid FirstName")?
                .to_string(),
            last_name: attrs
                .get("LastName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid LastName")?
                .to_string(),
            full_name: attrs
                .get("FullName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid FullName")?
                .to_string(),
            broadcast_name: attrs
                .get("BroadcastName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid BroadcastName")?
                .to_string(),
            headshot_url: attrs
                .get("HeadshotUrl")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid HeadshotUrl")?
                .to_string(),
            line: attrs
                .get("Line")
                .and_then(|v| v.as_u64())
                .ok_or("Missing or invalid Line")? as u8,
            public_id_right: attrs
                .get("PublicIdRight")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid PublicIdRight")?
                .to_string(),
            tla: attrs
                .get("Tla")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid Tla")?
                .to_string(),
            team_name: TeamName {
                value: attrs
                    .get("TeamName")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid TeamName")?
                    .to_string(),
            },
            reference: attrs
                .get("Reference")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid Reference")?
                .to_string(),
        }),
        _ => Err("Error parsing driver: attrs is not an Object".into()),
    }
}

fn parse_teams(val: &Value) -> HashMap<TeamName, Team> {
    let mut teams: HashMap<TeamName, Team> = HashMap::new();
    match val {
        Value::Object(map) => {
            for (_, attrs) in map.iter() {
                // Medical and safety cars don't have a team, so those fail to parse.
                // We just ignore them.
                if let Ok(t) = parse_team(attrs) {
                    teams.insert(t.name.clone(), t);
                };
            }
        }
        // TODO: Handle this properly
        _ => {}
    }
    teams
}

fn parse_team(val: &Value) -> Result<Team, Box<dyn std::error::Error>> {
    match val {
        Value::Object(attrs) => Ok(Team {
            name: TeamName {
                value: attrs
                    .get("TeamName")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid TeamName")?
                    .to_string(),
            },
            color: TeamColor {
                u32: attrs
                    .get("TeamColour")
                    .and_then(|v| v.as_str())
                    .and_then(|v| u32::from_str_radix(v, 16).ok())
                    .ok_or("Missing or invalid TeamColour")?,
            },
        }),
        _ => Err("Error parsing team from driver, should be a JSON Object".into()),
    }
}
