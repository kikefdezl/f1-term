use std::{fs::File, sync::Arc, time::Duration};

use crossterm::event::{self, Event};
use directories::ProjectDirs;
use f1_term_client::signalr::client::SignalRF1Client;
use f1_term_core::{
    client::{F1Client, TelemetryEvent},
    session::Session,
};
use f1_term_tui::tui::render;
use simplelog::{Config, LevelFilter, WriteLogger};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = if let Some(proj_dirs) = ProjectDirs::from("", "", "f1-term") {
        let dir = proj_dirs
            .state_dir()
            .unwrap_or_else(|| proj_dirs.data_local_dir());
        std::fs::create_dir_all(dir).unwrap();
        dir.join("f1-term.log")
    } else {
        std::path::PathBuf::from("f1-term.log")
    };

    let _ = WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create(&log_path).unwrap(),
    );

    let mut client = SignalRF1Client::new();
    client.connect().await?;

    let mut terminal = ratatui::init();

    let mut session_state: Option<Arc<Session>> = None;
    let mut render_interval = tokio::time::interval(Duration::from_millis(333));

    loop {
        tokio::select! {
            // Update data when telemetry arrives
            update = client.next_event() => {
                if let Some(TelemetryEvent::SessionUpdate(fs)) = update {
                    session_state = Some(fs);
                }
            }

            // Check for key press to exit and render
            _ = render_interval.tick() => {
                if event::poll(Duration::from_millis(0))? && let Event::Key(_) = event::read()? {
                        break;
                }

                if let Some(state) = session_state.as_ref()
                    && !state.drivers.is_empty()
                    && !state.teams.is_empty()
                {
                    terminal.draw(|frame| render(frame, state))?;
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}
