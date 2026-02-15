use f1_term_core::snapshot::FullSnapshot;
use ratatui::widgets::Paragraph;
use ratatui::{DefaultTerminal, Frame};

#[derive(Default)]
pub struct Tui {
    pub snapshot: FullSnapshot,
}

impl Tui {
    pub fn render(&self, frame: &mut Frame) {
        let text = self.snapshot.to_string();
        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, frame.area());
    }
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let tui = Tui::default();

    loop {
        terminal.draw(|frame| tui.render(frame))?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}
