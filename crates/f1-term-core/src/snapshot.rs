use super::driver::{Driver, DriverNumber};
use super::stint::Stints;
use super::team::{Team, TeamName};
use super::timing::LiveTiming;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct FullSnapshot {
    pub teams: HashMap<TeamName, Team>,
    pub drivers: HashMap<DriverNumber, Driver>,
    pub timing_data: HashMap<DriverNumber, LiveTiming>,
    pub stints: HashMap<DriverNumber, Stints>,
}
