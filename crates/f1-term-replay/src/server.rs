use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use log::{info, warn};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::player::Player;

const RECORD_SEPARATOR: &str = "\u{001E}";

pub struct AppState {
    pub player: Arc<RwLock<Player>>,
}

pub async fn start_server(player: Arc<RwLock<Player>>, port: u16) {
    let state = Arc::new(AppState { player });

    let app = Router::new()
        .route("/negotiate", any(negotiate_handler))
        .route("/", get(connect_handler))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("Replay server listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn negotiate_handler() -> impl IntoResponse {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::SET_COOKIE,
        "AWSALBCORS=mock_cookie; Path=/; SameSite=None; Secure"
            .parse()
            .unwrap(),
    );
    (
        headers,
        axum::Json(json!({
            "connectionToken": "mock_token",
            "connectionId": "mock_id",
            "negotiateVersion": 1,
            "availableTransports": [
                {
                    "transport": "WebSockets",
                    "transferFormats": [
                        "Text",
                        "Binary"
                    ]
                }
            ]
        })),
    )
}

async fn connect_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("Client connected to websocket");

    // 1. Wait for SignalR Core Handshake
    while let Some(msg) = socket.recv().await {
        if let Ok(Message::Text(_)) = msg {
            let response = format!("{}{}", json!({}), RECORD_SEPARATOR);
            socket.send(Message::Text(response.into())).await.ok();
            break;
        }
    }

    // 2. Wait for Subscribe
    while let Some(msg) = socket.recv().await {
        if let Ok(Message::Text(text)) = msg {
            let stripped = text.trim_end_matches(RECORD_SEPARATOR);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(stripped)
                && json.get("target").and_then(|t| t.as_str()) == Some("Subscribe")
            {
                info!("Client subscribed, sending canonical state");
                let player = state.player.read().await;

                let initial_state = json!({
                    "type": 3,
                    "invocationId": json.get("invocationId").unwrap_or(&json!("1")),
                    "result": player.canonical_state
                });

                let msg_str = format!("{}{}", initial_state, RECORD_SEPARATOR);
                if let Err(e) = socket.send(Message::Text(msg_str.into())).await {
                    warn!("Failed to send initial state: {}", e);
                }
                break;
            }
        }
    }

    let tick_rate = Duration::from_millis(100);
    let mut interval = tokio::time::interval(tick_rate);
    let mut last_seek_counter = state.player.read().await.seek_counter;

    loop {
        interval.tick().await;

        let mut player = state.player.write().await;

        if player.seek_counter != last_seek_counter {
            last_seek_counter = player.seek_counter;
            // Send full canonical state immediately using a mock type 3 message
            let state_payload = json!({
                "type": 3,
                "invocationId": "mock",
                "result": player.canonical_state
            });
            let msg_str = format!("{}{}", state_payload, RECORD_SEPARATOR);
            if let Err(e) = socket.send(Message::Text(msg_str.into())).await {
                warn!("Client disconnected during seek: {}", e);
                break;
            }
        }

        let messages = player.tick(tick_rate);

        if messages.is_empty() {
            continue;
        }

        let mut out_str = String::new();
        for msg in messages {
            let payload = json!({
                "type": 1,
                "target": "feed",
                "arguments": [msg.topic, msg.delta]
            });
            out_str.push_str(&payload.to_string());
            out_str.push_str(RECORD_SEPARATOR);
        }

        if let Err(e) = socket.send(Message::Text(out_str.into())).await {
            warn!("Client disconnected: {}", e);
            break;
        }
    }
}
