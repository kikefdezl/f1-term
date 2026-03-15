use std::collections::HashMap;

use f1_term_core::driver::DriverNumber;
use f1_term_core::stint::{Compound, Stint, Stints};
use log::warn;

use crate::parsing::stints::{RawDriverStints, RawStint};

impl TryFrom<&RawStint> for Stint {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: &RawStint) -> Result<Self, Self::Error> {
        let compound = match payload.Compound.as_str() {
            "SOFT" => Compound::Soft,
            "MEDIUM" => Compound::Medium,
            "HARD" => Compound::Hard,
            "INTERMEDIATE" => Compound::Intermediate,
            "WET" => Compound::Wet,
            _ => Compound::Unknown,
        };

        Ok(Stint {
            compound,
            lap_flags: payload.LapFlags,
            new: payload.New == "true",
            start_laps: payload.StartLaps,
            total_laps: payload.TotalLaps,
            tires_not_changed: payload.TyresNotChanged.parse().unwrap_or(0),
        })
    }
}

pub fn convert_stints(
    raw_stints_map: &HashMap<String, RawDriverStints>,
) -> HashMap<DriverNumber, Stints> {
    let mut stints_map: HashMap<DriverNumber, Stints> = HashMap::new();

    for (num_str, payload) in raw_stints_map {
        let Ok(number) = num_str.parse::<u8>() else {
            warn!("Failed to parse stint line {}", num_str);
            continue;
        };
        let driver_number = DriverNumber { value: number };

        let driver_stints: Stints = payload
            .Stints
            .iter()
            .filter_map(|s| match Stint::try_from(s) {
                Ok(stint) => Some(stint),
                Err(e) => {
                    warn!("Failed to convert stint for driver {}: {}", num_str, e);
                    None
                }
            })
            .collect();
        stints_map.insert(driver_number, driver_stints);
    }

    stints_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_stints() {
        let mut raw = HashMap::new();
        raw.insert(
            "1".to_string(),
            RawDriverStints {
                Stints: vec![
                    RawStint {
                        Compound: "SOFT".to_string(),
                        LapFlags: 0,
                        New: "true".to_string(),
                        StartLaps: 0,
                        TotalLaps: 15,
                        TyresNotChanged: "0".to_string(),
                    },
                    RawStint {
                        Compound: "MEDIUM".to_string(),
                        LapFlags: 0,
                        New: "false".to_string(),
                        StartLaps: 3,
                        TotalLaps: 25,
                        TyresNotChanged: "0".to_string(),
                    },
                ],
            },
        );

        let stints_map = convert_stints(&raw);
        assert_eq!(stints_map.len(), 1);

        let driver_number = DriverNumber { value: 1 };
        let stints = stints_map.get(&driver_number).unwrap();

        assert_eq!(stints.len(), 2);

        assert!(matches!(stints[0].compound, Compound::Soft));
        assert!(stints[0].new);
        assert_eq!(stints[0].start_laps, 0);
        assert_eq!(stints[0].total_laps, 15);

        assert!(matches!(stints[1].compound, Compound::Medium));
        assert!(!stints[1].new);
        assert_eq!(stints[1].start_laps, 3);
        assert_eq!(stints[1].total_laps, 25);
    }

    #[test]
    fn test_stints_missing_fields() {
        let mut raw = HashMap::new();
        raw.insert(
            "44".to_string(),
            RawDriverStints {
                Stints: vec![RawStint {
                    Compound: "SOFT".to_string(),
                    New: "true".to_string(),
                    StartLaps: 0,
                    TotalLaps: 5,
                    ..Default::default()
                }],
            },
        );

        let map = convert_stints(&raw);
        let stints = map.get(&DriverNumber { value: 44 }).unwrap();

        assert_eq!(stints.len(), 1);
        assert_eq!(stints[0].tires_not_changed, 0);
        assert_eq!(stints[0].lap_flags, 0);
    }
}
