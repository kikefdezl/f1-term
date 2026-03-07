use std::collections::HashMap;

use f1_term_core::{
    driver::{Driver, DriverNumber},
    team::TeamName,
};
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
