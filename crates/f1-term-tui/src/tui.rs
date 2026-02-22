use f1_term_core::session::Session;
use ratatui::Frame;

use crate::table::{Table, TableData, TableDataArgs};

pub fn render(frame: &mut Frame, session: &Session) {
    let table_datas = {
        let mut tds = Vec::new();
        for participant in session.leaderboard() {
            let args = TableDataArgs {
                driver: participant.driver,
                team: participant.team,
                live_timing: participant.timing,
                stints: participant.stints,
            };
            tds.push(TableData::from(&args));
        }
        tds
    };
    let table = Table::new(table_datas);
    frame.render_widget(table, frame.area());
}
