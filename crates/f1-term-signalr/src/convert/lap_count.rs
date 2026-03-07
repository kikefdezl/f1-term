use f1_term_core::laps::Laps;

use crate::parsing::lap_count::RawLapCount;

impl From<&RawLapCount> for Laps {
    fn from(value: &RawLapCount) -> Self {
        Laps {
            current: value.CurrentLap,
            total: value.TotalLaps,
        }
    }
}

pub fn convert_lap_count(raw: &RawLapCount) -> Laps {
    Laps::from(raw)
}
