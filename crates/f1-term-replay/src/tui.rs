use std::sync::Arc;

use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};
use tokio::sync::RwLock;

use crate::Result;
use crate::api::client::F1ApiClient;
use crate::api::models::SessionIndex;
use crate::player::Player;

pub async fn select_session() -> Result<Option<SessionIndex>> {
    let api = F1ApiClient::new();

    let mut all_sessions = Vec::new();

    for year in [2024, 2025, 2026] {
        match api.get_index(year).await {
            Ok(index) => {
                for meeting in &index.meetings {
                    for session in &meeting.sessions {
                        all_sessions.push(session.clone());
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to fetch index for year {}: {}", year, e);
            }
        }
    }

    all_sessions.sort_by(|a, b| b.start_date.cmp(&a.start_date));

    if all_sessions.is_empty() {
        return Err("No sessions found from the F1 API.".into());
    }

    let mut terminal = ratatui::init();
    let res = run_selection_tui(&mut terminal, all_sessions);
    ratatui::restore();

    res
}

fn run_selection_tui<B: Backend>(
    terminal: &mut Terminal<B>,
    sessions: Vec<SessionIndex>,
) -> Result<Option<SessionIndex>>
where
    <B as Backend>::Error: std::error::Error + Send + Sync + 'static,
{
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.area());

            let header = Paragraph::new("Select a Session to Replay")
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL).title("F1 Replay"));
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = sessions
                .iter()
                .map(|s| ListItem::new(format!("{} - {} ({})", s.start_date, s.name, s.path)))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Sessions"))
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[1], &mut list_state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i >= sessions.len().saturating_sub(1) {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = match list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                sessions.len().saturating_sub(1)
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    list_state.select(Some(i));
                }
                KeyCode::Enter => {
                    if let Some(i) = list_state.selected() {
                        return Ok(Some(sessions[i].clone()));
                    }
                }
                _ => {}
            }
        }
    }
}

pub async fn run_playback_tui(player: Arc<RwLock<Player>>) -> Result<()> {
    let mut terminal = ratatui::init();

    loop {
        let (current_time, duration, is_playing) = {
            let p = player.read().await;
            (p.current_time, p.duration, p.is_playing)
        };

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(7), Constraint::Min(0)].as_ref())
                .split(f.area());

            let status = if is_playing { "PLAYING" } else { "PAUSED" };
            let progress = if duration.as_secs() > 0 {
                current_time.as_secs_f64() / duration.as_secs_f64()
            } else {
                0.0
            };

            let info_text = format!(
                "Status: {}\nSession Timer: {:02}:{:02}:{:02} / {:02}:{:02}:{:02}\n\nControls:\n  [Space / Down] Pause/Play\n  [Left] Seek -30s\n  [Right] Seek +30s\n  [Q / Esc] Quit",
                status,
                current_time.as_secs() / 3600,
                (current_time.as_secs() / 60) % 60,
                current_time.as_secs() % 60,
                duration.as_secs() / 3600,
                (duration.as_secs() / 60) % 60,
                duration.as_secs() % 60,
            );

            let paragraph = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("F1 Replay Player"))
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
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char(' ') | KeyCode::Down => p.toggle_pause(),
                KeyCode::Left | KeyCode::Char('h') => p.seek_by(-30),
                KeyCode::Right | KeyCode::Char('l') => p.seek_by(30),
                _ => {}
            }
        }
    }

    ratatui::restore();
    Ok(())
}
