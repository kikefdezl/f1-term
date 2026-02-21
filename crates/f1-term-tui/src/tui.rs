use f1_term_core::snapshot::FullSnapshot;
use ratatui::Frame;

use crate::table::{Table, TableData, TableDataArgs};

pub fn render(frame: &mut Frame, snapshot: &FullSnapshot) {
    let table_datas = {
        let mut tds = Vec::new();
        for driver in snapshot.drivers.values() {
            let team = snapshot
                .teams
                .get(&driver.team_name)
                .expect("Team should be in snapshot");
            let live_timing = snapshot.timing_data.get(&driver.number);
            let args = TableDataArgs {
                driver,
                team,
                live_timing,
            };
            tds.push(TableData::from(&args));
        }
        tds
    };
    let table = Table::new(table_datas);
    frame.render_widget(table, frame.area());
}
