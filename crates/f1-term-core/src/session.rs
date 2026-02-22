use std::collections::HashMap;

use super::{
    driver::{Driver, DriverNumber},
    stint::Stints,
    team::{Team, TeamName},
    timing::LiveTiming,
};

#[derive(Debug, Default)]
pub struct Session {
    pub teams: HashMap<TeamName, Team>,
    pub drivers: HashMap<DriverNumber, Driver>,
    pub timing_data: HashMap<DriverNumber, LiveTiming>,
    pub stints: HashMap<DriverNumber, Stints>,
}

#[derive(Debug)]
pub struct ParticipantContext<'a> {
    pub driver: &'a Driver,
    pub team: &'a Team,
    pub timing: Option<&'a LiveTiming>,
    pub stints: Option<&'a Stints>,
}

impl Session {
    /// Returns the active grid joined together and ordered by their live timing position.
    /// If timing data is missing, falls back to their natural grid line order.
    pub fn leaderboard(&self) -> Vec<ParticipantContext<'_>> {
        let mut contexts = Vec::with_capacity(self.drivers.len());

        for driver in self.drivers.values() {
            let team = self
                .teams
                .get(&driver.team_name)
                .expect("Driver must have an associated team");

            contexts.push(ParticipantContext {
                driver,
                team,
                timing: self.timing_data.get(&driver.number),
                stints: self.stints.get(&driver.number),
            });
        }

        // Sort by current race position (timing.position) then grid position (driver.line)
        contexts.sort_by(|a, b| {
            let pos_a = a.timing.map(|t| t.position).unwrap_or(u8::MAX);
            let pos_b = b.timing.map(|t| t.position).unwrap_or(u8::MAX);

            if pos_a != pos_b {
                pos_a.cmp(&pos_b)
            } else {
                let line_a = a.driver.line.unwrap_or(u8::MAX);
                let line_b = b.driver.line.unwrap_or(u8::MAX);
                line_a.cmp(&line_b)
            }
        });

        contexts
    }
}
