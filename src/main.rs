use f1_term_client::signalr::client::SignalRF1Client;
use f1_term_core::client::{F1Client, TelemetryEvent};
use f1_term_tui::tui::Tui;

use crossterm::event::{self, Event};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SignalRF1Client::new();
    client.connect().await?;
    let mut terminal = ratatui::init();
    let mut tui = Tui::default();

    loop {
        tokio::select! {
            // Wait for telemetry update
            update = client.next_event() => {
                if let Some(TelemetryEvent::Full(fs)) = update {
                    tui.snapshot = fs;
                }
            }

            // Check for key press (in async context)
            _ = tokio::time::sleep(Duration::from_millis(16)) => {
                if event::poll(Duration::from_millis(0))? {
                    if let Event::Key(_) = event::read()? {
                        break;
                    }
                }
            }
        }

        terminal.draw(|frame| tui.render(frame))?;
    }

    ratatui::restore();
    Ok(())
}
