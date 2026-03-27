use super::driver::DriverNumber;
use crate::lap_time::LapTime;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LiveTiming {
    pub driver_number: DriverNumber,
    pub position: u8,
    pub status: u32,
    pub retired: bool,
    pub stopped: bool,
    pub lap_data: LapData,
    pub pit_data: PitData,
    pub time_diffs: TimeDiffs,
    pub quali_stats: Option<QualiStats>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LapData {
    pub best_lap: BestLap,
    pub last_lap: LastLap,
    pub number_of_laps: Option<u8>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BestLap {
    pub time: Option<LapTime>,
    pub overall_fastest: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LastLap {
    pub overall_fastest: bool,
    pub personal_fastest: bool,
    pub status: u32,
    pub time: Option<LapTime>,
    pub sectors: Vec<Sector>,
    pub show_position: bool,
    pub speeds: Speeds,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PitData {
    pub in_pit: bool,
    pub pit_out: bool,
    pub number_of_pit_stops: Option<u8>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct QualiStats {
    pub cutoff: Option<bool>,
    pub knocked_out: Option<bool>,
    pub q1_diffs: Option<TimeDiffs>,
    pub q2_diffs: Option<TimeDiffs>,
    pub q3_diffs: Option<TimeDiffs>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Sector {
    pub overall_fastest: bool,
    pub personal_fastest: bool,
    pub segments: Vec<Segment>,
    pub status: u32,
    pub stopped: bool,
    /// value is None if driver is on their next lap
    pub value: Option<String>,
    pub previous_value: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Segment {
    pub status: SegmentStatus,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum SegmentStatus {
    #[default]
    None,
    Normal,
    OverallFastest,
    PersonalFastest,
    Aborted,
    InPit,
    Unknown,
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
    pub status: u32,
    pub value: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TimeDiffs {
    pub to_fastest: Option<String>,
    pub to_position_ahead: Option<String>,
}
