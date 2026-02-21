use super::driver::DriverNumber;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LiveTiming {
    pub driver_number: DriverNumber,
    pub best_lap_time: Option<String>,
    pub in_pit: bool,
    pub pit_out: bool,
    pub last_lap: LastLap,
    pub position: u8,
    pub retired: bool,
    pub status: u8,
    pub stopped: bool,
    pub time_diff_to_fastest: String,
    pub time_diff_to_position_ahead: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LastLap {
    pub overall_fastest: bool,
    pub personal_fastest: bool,
    pub status: u8,
    pub time: Option<String>,
    pub sectors: Vec<Sector>,
    pub show_position: bool,
    pub speeds: Speeds,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Sector {
    pub overall_fastest: bool,
    pub personal_fastest: bool,
    pub segments: Vec<Segment>,
    pub status: u8,
    pub stopped: bool,
    pub value: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Segment {
    pub status: u8,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Speeds {
    pub fl: Speed,
    pub i1: Speed,
    pub i2: Speed,
    pub st: Speed,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Speed {
    pub overall_fastest: bool,
    pub personal_fastest: bool,
    pub status: u8,
    pub value: String,
}
