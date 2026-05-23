use chrono::{DateTime, Duration, Utc};

use crate::coord::Coord;

#[derive(Clone, Default, Debug)]
pub struct DriverPosition {
    pub coord: Coord,

    pub _last_sector_started_at: DateTime<Utc>,
    pub _sector_estimate: Duration,
}

impl DriverPosition {
    pub fn new(coord: Coord) -> DriverPosition {
        DriverPosition {
            coord,
            ..Default::default()
        }
    }
}
