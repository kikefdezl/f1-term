pub mod api;
pub mod player;
pub mod server;
pub mod tui;

use std::sync::Arc;

use api::client::F1ApiClient;
use log::error;
use player::Player;
use tokio::sync::RwLock;

use crate::tui::select_session;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Loading F1 sessions...");
    let session = match select_session().await {
        Ok(Some(s)) => s,
        Ok(None) => {
            println!("No session selected. Exiting.");
            return Ok(());
        }
        Err(e) => {
            error!("UI Error: {}", e);
            return Err(e);
        }
    };

    println!("Selected session: {}", session.name);
    println!("Loading streams... This might take a while.");

    let api = F1ApiClient::new();
    let root_index = api.get_session_index(&session.path).await?;

    let mut base_state = serde_json::json!({});

    // Fetch base state for all feeds and assemble the canonical state.
    for (feed_name, feed) in &root_index.feeds {
        if let Ok(json) = api.get_json(&session.path, &feed.key_frame_path).await {
            base_state[feed_name] = json;
        }
    }

    let player = Arc::new(RwLock::new(Player::new(base_state)));

    let mut tasks = Vec::new();

    // Fetch and parse streams concurrently
    for (feed_name, feed) in root_index.feeds {
        let api_clone = api.clone();
        let session_path = session.path.clone();
        let player_clone = Arc::clone(&player);

        let task = tokio::task::spawn(async move {
            match api_clone
                .get_json_stream(&session_path, &feed.stream_path)
                .await
            {
                Ok(stream_data) => {
                    let mut p = player_clone.write().await;
                    p.parse_stream(&feed_name, &stream_data);
                }
                Err(e) => {
                    log::error!("Failed to fetch stream {}: {}", feed_name, e);
                }
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    println!("Streams loaded! Starting replay server on port 5000");
    println!("Run `f1-term --replay` in another terminal to connect.");

    // Start server in background
    let server_player = Arc::clone(&player);
    tokio::spawn(async move {
        server::start_server(server_player, 5000).await;
    });

    // Run Playback TUI in foreground
    if let Err(e) = tui::run_playback_tui(player).await {
        error!("Playback UI error: {}", e);
    }

    Ok(())
}
