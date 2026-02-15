use f1_term_core::snapshot::FullSnapshot;
use ratatui::Frame;

use crate::table::{Table, TableData};

pub fn render(frame: &mut Frame, snapshot: &FullSnapshot) {
    let table_datas = {
        let mut tds = Vec::new();
        for driver in snapshot.drivers.values() {
            let team = snapshot
                .teams
                .get(&driver.team_name)
                .expect("Team should be in snapshot");
            tds.push(TableData::from_driver_team(driver, team));
        }
        tds
    };
    let table = Table::new(table_datas);
    frame.render_widget(table, frame.area());
}
