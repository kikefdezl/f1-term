use std::collections::HashMap;

use f1_term_core::{
    driver::DriverNumber,
    stint::{Compound, Stint, Stints},
};
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
