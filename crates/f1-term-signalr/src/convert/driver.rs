use std::collections::HashMap;

use f1_term_core::driver::{Driver, DriverNumber};
use f1_term_core::team::TeamName;
use log::{info, warn};

use crate::parsing::driver_list::RawDriver;

impl TryFrom<&RawDriver> for Driver {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: &RawDriver) -> Result<Self, Self::Error> {
        Ok(Driver {
            number: DriverNumber {
                value: payload.RacingNumber.parse()?,
            },
            first_name: payload.FirstName.clone(),
            last_name: payload.LastName.clone(),
            full_name: payload.FullName.clone(),
            broadcast_name: payload.BroadcastName.clone(),
            headshot_url: payload.HeadshotUrl.clone(),
            line: Some(payload.Line),
            public_id_right: payload.PublicIdRight.clone(),
            tla: payload.Tla.clone(),
            team_name: TeamName {
                value: payload.TeamName.clone(),
            },
            reference: payload.Reference.clone(),
            position: None,  // this has to be aggregated later
        })
    }
}

pub fn convert_drivers(raw_drivers: &HashMap<String, RawDriver>) -> HashMap<DriverNumber, Driver> {
    let mut drivers = HashMap::new();

    for (num_str, payload) in raw_drivers {
        let Ok(number) = num_str.parse::<u8>() else {
            warn!("Failed to parse number for car {}", num_str);
            continue;
        };

        match Driver::try_from(payload) {
            Ok(driver) => {
                drivers.insert(DriverNumber { value: number }, driver);
            }
            Err(e) => {
                info!("Failed to parse driver payload for car {}: {}", number, e);
            }
        }
    }

    drivers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_drivers() {
        let mut raw_drivers = HashMap::new();
        raw_drivers.insert(
            "1".to_string(),
            RawDriver {
                RacingNumber: "1".to_string(),
                FirstName: "Max".to_string(),
                LastName: "Verstappen".to_string(),
                FullName: "Max VERSTAPPEN".to_string(),
                BroadcastName: "M VERSTAPPEN".to_string(),
                HeadshotUrl: "url".to_string(),
                Line: 1,
                PublicIdRight: "id".to_string(),
                Tla: "VER".to_string(),
                TeamName: "Red Bull Racing".to_string(),
                Reference: "ref".to_string(),
                TeamColour: "3671C6".to_string(),
            },
        );

        let drivers = convert_drivers(&raw_drivers);
        assert_eq!(drivers.len(), 1);

        let driver = drivers.get(&DriverNumber { value: 1 }).unwrap();
        assert_eq!(driver.first_name, "Max");
        assert_eq!(driver.last_name, "Verstappen");
        assert_eq!(driver.tla, "VER");
        assert_eq!(driver.team_name.value, "Red Bull Racing");
    }
}
