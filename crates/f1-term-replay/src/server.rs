use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use log::{info, warn};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::player::Player;

pub struct AppState {
    pub player: Arc<RwLock<Player>>,
}

pub async fn start_server(player: Arc<RwLock<Player>>, port: u16) {
    let state = Arc::new(AppState { player });

    let app = Router::new()
        .route("/negotiate", get(negotiate_handler))
        .route("/connect", get(connect_handler))
        .route("/start", get(start_handler))
        .with_state(state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("Replay server listening on port {}", port);
    axum::serve(listener, app).await.unwrap();
}

async fn negotiate_handler() -> impl IntoResponse {
    axum::Json(json!({
        "ConnectionToken": "mock_token",
        "ConnectionId": "mock_id",
        "KeepAliveTimeout": 20.0,
        "DisconnectTimeout": 30.0,
        "ConnectionTimeout": 110.0,
        "TryWebSockets": true,
        "ProtocolVersion": "1.5",
        "TransportConnectTimeout": 5.0,
        "LongPollDelay": 0.0
    }))
}

async fn connect_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn start_handler() -> impl IntoResponse {
    axum::Json(json!({"Response": "started"}))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    info!("Client connected to websocket");

    // Send the initial canonical state.
    // SignalR usually sends `{"R": { ...canonical state ... }}` as a response to a `Subscribe` message.
    while let Some(msg) = socket.recv().await {
        if let Ok(Message::Text(text)) = msg
            && let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
            && json.get("M").and_then(|m| m.as_str()) == Some("Subscribe")
        {
            info!("Client subscribed, sending canonical state");
            let player = state.player.read().await;
            let initial_state = json!({
                "I": json.get("I").unwrap_or(&json!("1")),
                "R": player.canonical_state
            });
            if let Err(e) = socket
                .send(Message::Text(initial_state.to_string().into()))
                .await
            {
                warn!("Failed to send initial state: {}", e);
            }
            break;
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
            // Send full canonical state immediately
            let state_payload = json!({
                "R": player.canonical_state
            });
            if let Err(e) = socket
                .send(Message::Text(state_payload.to_string().into()))
                .await
            {
                warn!("Client disconnected during seek: {}", e);
                break;
            }
        }

        let messages = player.tick(tick_rate);

        if messages.is_empty() {
            continue;
        }

        let m_array: Vec<_> = messages
            .into_iter()
            .map(|msg| {
                json!({
                    "H": "Streaming",
                    "M": "feed",
                    "A": [msg.topic, msg.delta]
                })
            })
            .collect();

        let payload = json!({
            "M": m_array
        });

        if let Err(e) = socket.send(Message::Text(payload.to_string().into())).await {
            warn!("Client disconnected: {}", e);
            break;
        }
    }
}
