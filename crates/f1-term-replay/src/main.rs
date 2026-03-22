pub mod api;
pub mod app;
pub mod player;
pub mod server;

use crate::app::App;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new();
    app.init().await;
    let mut terminal = ratatui::init();
    app.run(&mut terminal).await?;
    ratatui::restore();
    Ok(())
}
