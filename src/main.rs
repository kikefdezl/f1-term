use std::fs::File;

use directories::ProjectDirs;
use f1_term_client::signalr::client::SignalRF1Client;
use f1_term_core::client::F1Client;
use f1_term_tui::app::App;
use simplelog::{Config, LevelFilter, WriteLogger};

const APP: &str = "f1-term";
const LOGFILE: &str = "f1-term.log";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = if let Some(proj_dirs) = ProjectDirs::from("", "", APP) {
        let dir = proj_dirs
            .state_dir()
            .unwrap_or_else(|| proj_dirs.data_local_dir());
        std::fs::create_dir_all(dir).unwrap();
        dir.join(LOGFILE)
    } else {
        std::path::PathBuf::from(LOGFILE)
    };

    let _ = WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create(&log_path).unwrap(),
    );

    let mut client = SignalRF1Client::new();
    client.connect().await?;

    let mut terminal = ratatui::init();

    let mut app = App::new(client);
    let res = app.run(&mut terminal).await;

    ratatui::restore();

    res
}
