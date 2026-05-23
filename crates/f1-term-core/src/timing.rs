use crate::driver::DriverNumber;
use crate::gap::Gap;
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

impl LastLap {
    /// Aproximation of the lap completion percentage based on the statuses of the
    /// segments. This is not a very accurate value.
    pub fn percentage_lap_done(&self) -> Option<f64> {
        let total = self.total_segments();
        if total == 0 {
            return None;
        }
        self.current_segment().map(|s| s as f64 / total as f64)
    }

    pub fn current_segment(&self) -> Option<usize> {
        let mut total = 0;
        let mut started = false;

        for sector in &self.sectors {
            match sector.current_segment() {
                Some(s) => {
                    started = true;
                    total += s;
                    if s < sector.segments.len() {
                        return Some(total);
                    }
                }
                None => {
                    if started {
                        return Some(total);
                    } else {
                        return None;
                    }
                }
            }
        }

        if started { Some(total) } else { None }
    }

    pub fn total_segments(&self) -> usize {
        self.sectors.iter().map(|s| s.segments.len()).sum()
    }
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
    pub value: Option<LapTime>,
    pub previous_value: Option<LapTime>,
}

impl Sector {
    pub fn current_segment(&self) -> Option<usize> {
        if self.segments.is_empty() {
            return None;
        }
        if let SegmentStatus::None = self.segments[0].status {
            return None;
        }
        let mut i = 0;
        while i < self.segments.len() && self.segments[i].status != SegmentStatus::None {
            i += 1;
        }
        Some(i)
    }
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
    pub to_fastest: Option<Gap>,
    pub to_position_ahead: Option<Gap>,
}
