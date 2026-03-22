mod args;
mod logger;

use f1_term_core::telemetry_engine::TelemetryEngine;
use f1_term_multiviewer::client::MultiviewerClient;
use f1_term_signalr::client::SignalRF1Client;
use f1_term_tui::app::App;
use tokio::sync::mpsc;

const REPLAY_URL: &str = "localhost:5000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();

    let log_dir = logger::init();

    let mut telemetry_provider =
        SignalRF1Client::new().with_log_dir(log_dir.to_string_lossy().into_owned());

    if args.replay {
        telemetry_provider = telemetry_provider.with_base_url(REPLAY_URL.to_string());
    }

    let circuit_provider = MultiviewerClient::new();

    let mut engine = TelemetryEngine::new(telemetry_provider, circuit_provider);
    engine.connect().await?;

    let state = engine.get_state();

    let (eng_tx, eng_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        engine.run(eng_rx).await;
    });

    let mut terminal = ratatui::init();

    let mut app = App::new(state, eng_tx);
    let res = app.run(&mut terminal).await;

    ratatui::restore();

    res
}
