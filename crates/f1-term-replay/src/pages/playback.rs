use std::sync::Arc;

use crossterm::event::{self, Event, KeyCode};
use log::error;
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::Result;
use crate::api::client::F1ApiClient;
use crate::api::models::SessionIndex;
use crate::app::Action;
use crate::pages::{ActivePage, Page};
use crate::player::{Player, TimelineStatus};
use crate::server::start_server;

#[derive(Default)]
pub struct PlaybackPage {
    api: F1ApiClient,
    player: Option<Arc<RwLock<Player>>>,
    session: Option<SessionIndex>,
    loading_task: Option<JoinHandle<()>>,
    server_task: Option<JoinHandle<()>>,
}

impl PlaybackPage {
    pub fn new() -> PlaybackPage {
        PlaybackPage {
            api: F1ApiClient::new(),
            player: None,
            session: None,
            loading_task: None,
            server_task: None,
        }
    }

    pub fn start(&mut self, session: SessionIndex) {
        self.stop_tasks();

        self.session = Some(session.clone());
        let player = Arc::new(RwLock::new(Player::new()));
        self.player = Some(Arc::clone(&player));

        let loading_player = Arc::clone(&player);
        let loading_api = self.api.clone();
        self.loading_task = Some(tokio::spawn(async move {
            load_streams(loading_api, loading_player, session)
                .await
                .expect("Streams failed to load");
        }));

        let server_player = Arc::clone(&player);
        self.server_task = Some(tokio::spawn(async move {
            start_server(server_player, 5000).await;
        }));
    }

    pub fn stop_tasks(&mut self) {
        if let Some(task) = self.loading_task.take() {
            task.abort();
        }
        if let Some(task) = self.server_task.take() {
            task.abort();
        }
    }
}

impl Page for PlaybackPage {
    async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<Action> {
        let player = match &self.player {
            Some(p) => Arc::clone(p),
            None => return Ok(Action::Activate(ActivePage::Selection)),
        };

        loop {
            if !matches!(player.read().await.timeline.status, TimelineStatus::Loaded) {
                terminal.draw(|f| {
                    let text = "Loading stream...";
                    let paragraph = Paragraph::new(text)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("F1 Replay Player"),
                        )
                        .style(Style::default().fg(Color::Cyan));
                    f.render_widget(paragraph, f.area());
                })?;

                if event::poll(std::time::Duration::from_millis(50))?
                    && let Event::Key(key) = event::read()?
                {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.stop_tasks();
                            return Ok(Action::Activate(ActivePage::Selection));
                        }
                        _ => {}
                    }
                }
                continue;
            }

            let (current_time, duration, is_playing) = {
                let p = player.read().await;
                (p.current_time, p.duration, p.is_playing)
            };

            let session_path = self
                .session
                .as_ref()
                .map(|s| s.path.clone())
                .unwrap_or_else(|| "Unknown Session".to_string());

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Length(12), Constraint::Length(5)].as_ref())
                    .split(f.area());

                let status = if is_playing { "PLAYING" } else { "PAUSED" };
                let progress = if duration.as_secs() > 0 {
                    current_time.as_secs_f64() / duration.as_secs_f64()
                } else {
                    0.0
                };

                let info_text = format!(
                    "Session: {}\n\
                     Status: {}\n\
                     Session Timer: {:02}:{:02}:{:02} / {:02}:{:02}:{:02}\n\
                     \n\
                     Controls:\n\
                     [Space / Down] Pause/Play\n\
                     [Left] Seek -30s\n\
                     [Right] Seek +30s\n\
                     [S] Select another Session\n\
                     [Q / Esc] Quit",
                    session_path,
                    status,
                    current_time.as_secs() / 3600,
                    (current_time.as_secs() / 60) % 60,
                    current_time.as_secs() % 60,
                    duration.as_secs() / 3600,
                    (duration.as_secs() / 60) % 60,
                    duration.as_secs() % 60,
                );

                let paragraph = Paragraph::new(info_text)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("F1 Replay Player"),
                    )
                    .style(Style::default().fg(Color::Cyan));
                f.render_widget(paragraph, chunks[0]);

                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Progress"))
                    .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
                    .ratio(progress.clamp(0.0, 1.0));
                f.render_widget(gauge, chunks[1]);
            })?;

            if event::poll(std::time::Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
            {
                let mut p = player.write().await;
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.stop_tasks();
                        return Ok(Action::Quit);
                    }
                    KeyCode::Char('s') => {
                        self.stop_tasks();
                        return Ok(Action::Activate(ActivePage::Selection));
                    }
                    KeyCode::Char(' ') | KeyCode::Down => p.toggle_pause(),
                    KeyCode::Left | KeyCode::Char('h') => p.seek_by(-30),
                    KeyCode::Right | KeyCode::Char('l') => p.seek_by(30),
                    _ => {}
                }
            }
        }
    }
}

pub async fn load_streams(
    api: F1ApiClient,
    player: Arc<RwLock<Player>>,
    session: SessionIndex,
) -> Result<()> {
    let root_index = api.get_session_index(&session.path).await?;

    let mut base_state = serde_json::json!({});
    for (feed_name, feed) in &root_index.feeds {
        if let Ok(json) = api.get_json(&session.path, &feed.key_frame_path).await {
            base_state[feed_name] = json;
        }
    }

    {
        let mut p = player.write().await;
        p.init_state(base_state);
        p.timeline.mark(TimelineStatus::Loading);
    }

    let mut tasks = Vec::new();

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
                    error!("Failed to fetch stream {}: {}", feed_name, e);
                }
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }

    {
        let mut p = player.write().await;
        p.timeline.mark(TimelineStatus::Loaded);
    }
    Ok(())
}
