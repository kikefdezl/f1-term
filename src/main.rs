use std::fs::File;

use directories::ProjectDirs;
use f1_term_core::telemetry_engine::TelemetryEngine;
use f1_term_multiviewer::client::MultiviewerClient;
use f1_term_signalr::client::SignalRF1Client;
use f1_term_tui::app::App;
use simplelog::{Config, LevelFilter, WriteLogger};
use tokio::sync::mpsc;

const APP: &str = "f1-term";
const LOGFILE: &str = "f1-term.log";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = if let Some(proj_dirs) = ProjectDirs::from("", "", APP) {
        let dir = proj_dirs
            .state_dir()
            .unwrap_or_else(|| proj_dirs.data_local_dir());
        std::fs::create_dir_all(dir).unwrap();
        dir.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    };

    let log_path = log_dir.join(LOGFILE);

    let _ = WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create(&log_path).unwrap(),
    );

    let telemetry_provider =
        SignalRF1Client::new().with_log_dir(log_dir.to_string_lossy().into_owned());
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
