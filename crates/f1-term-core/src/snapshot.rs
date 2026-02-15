use super::driver::{Driver, DriverNumber};
use super::team::{Team, TeamName};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Default)]
pub struct FullSnapshot {
    pub teams: HashMap<TeamName, Team>,
    pub drivers: HashMap<DriverNumber, Driver>,
}

impl Display for FullSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for team in self.teams.values() {
            writeln!(f, "{}", team)?;
        }
        for driver in self.drivers.values() {
            writeln!(f, "{}", driver)?;
        }
        Ok(())
    }
}
