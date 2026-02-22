use f1_term_client::signalr::client::SignalRF1Client;
use f1_term_core::client::{F1Client, TelemetryEvent};
use f1_term_core::snapshot::FullSnapshot;
use f1_term_tui::tui::render;

use crossterm::event::{self, Event};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SignalRF1Client::new();
    client.connect().await?;

    let mut terminal = ratatui::init();

    let mut last_snap: Option<FullSnapshot> = None;
    let mut render_interval = tokio::time::interval(Duration::from_millis(333));

    loop {
        tokio::select! {
            // Update data when telemetry arrives
            update = client.next_event() => {
                if let Some(TelemetryEvent::Full(fs)) = update {
                    last_snap = Some(fs);
                }
            }

            // Check for key press to exit and render
            _ = render_interval.tick() => {
                if event::poll(Duration::from_millis(0))? && let Event::Key(_) = event::read()? {
                        break;
                }

                if let Some(ls) = last_snap.as_mut()
                    && !ls.drivers.is_empty()
                    && !ls.teams.is_empty()
                {
                    terminal.draw(|frame| render(frame, ls))?;
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}
