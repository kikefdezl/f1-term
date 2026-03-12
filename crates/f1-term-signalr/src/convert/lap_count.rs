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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_lap_count() {
        let raw = RawLapCount {
            CurrentLap: 5,
            TotalLaps: 50,
        };

        let laps = convert_lap_count(&raw);

        assert_eq!(laps.current, 5);
        assert_eq!(laps.total, 50);
    }
}
