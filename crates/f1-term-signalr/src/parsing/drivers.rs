use std::collections::HashMap;

use f1_term_core::{
    driver::{Driver, DriverNumber},
    team::TeamName,
};
use log::info;
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct DriverPayload {
    RacingNumber: String,
    FirstName: String,
    LastName: String,
    FullName: String,
    BroadcastName: String,
    HeadshotUrl: String,
    Line: u8,
    PublicIdRight: String,
    Tla: String,
    TeamName: String,
    Reference: String,
}

impl TryFrom<DriverPayload> for Driver {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: DriverPayload) -> Result<Self> {
        Ok(Driver {
            number: DriverNumber {
                value: payload.RacingNumber.parse()?,
            },
            first_name: payload.FirstName,
            last_name: payload.LastName,
            full_name: payload.FullName,
            broadcast_name: payload.BroadcastName,
            headshot_url: payload.HeadshotUrl,
            line: Some(payload.Line),
            public_id_right: payload.PublicIdRight,
            tla: payload.Tla,
            team_name: TeamName {
                value: payload.TeamName,
            },
            reference: payload.Reference,
        })
    }
}

pub fn parse_drivers(val: &Value) -> Result<HashMap<DriverNumber, Driver>> {
    let mut drivers: HashMap<DriverNumber, Driver> = HashMap::new();
    match val {
        Value::Object(map) => {
            for (num, attrs) in map.iter() {
                let number: u8 = match num.parse() {
                    Ok(n) => n,
                    // Some non-grid cars have non-digits or numbers above 255. Ignore.
                    Err(_) => continue,
                };
                let driver_number = DriverNumber { value: number };

                // Medical and safety cars don't have all fields, so those fail to parse.
                // We just ignore them too.
                match DriverPayload::deserialize(attrs) {
                    Ok(payload) => match Driver::try_from(payload) {
                        Ok(d) => {
                            drivers.insert(driver_number, d);
                        }
                        Err(e) => {
                            info!("Failed to convert driver payload {}: {}", number, e);
                        }
                    },
                    Err(e) => {
                        info!("Failed to parse driver payload for {}: {}", number, e);
                    }
                }
            }
        }
        _ => return Err("Drivers value is not a JSON object".into()),
    }
    Ok(drivers)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_drivers() {
        let val = json!({
            "1": {
                "RacingNumber": "1",
                "FirstName": "Max",
                "LastName": "Verstappen",
                "FullName": "Max VERSTAPPEN",
                "BroadcastName": "M VERSTAPPEN",
                "HeadshotUrl": "https://example.com/max.png",
                "Line": 1,
                "PublicIdRight": "something",
                "Tla": "VER",
                "TeamName": "Red Bull Racing",
                "Reference": "MAXVER01"
            },
            "16": {
                "RacingNumber": "16",
                "FirstName": "Charles",
                "LastName": "Leclerc",
                "FullName": "Charles LECLERC",
                "BroadcastName": "C LECLERC",
                "HeadshotUrl": "https://example.com/charles.png",
                "Line": 2,
                "PublicIdRight": "something_else",
                "Tla": "LEC",
                "TeamName": "Ferrari",
                "Reference": "CHALEC01"
            },
            "SC": {
                "RacingNumber": "SC",
                "SomeOtherField": "Medical Car"
            }
        });

        let drivers = parse_drivers(&val).unwrap();

        assert_eq!(drivers.len(), 2);

        let driver_1 = drivers.get(&DriverNumber { value: 1 }).unwrap();
        assert_eq!(driver_1.first_name, "Max");
        assert_eq!(driver_1.last_name, "Verstappen");
        assert_eq!(driver_1.tla, "VER");
        assert_eq!(driver_1.team_name.value, "Red Bull Racing");

        let driver_16 = drivers.get(&DriverNumber { value: 16 }).unwrap();
        assert_eq!(driver_16.first_name, "Charles");
        assert_eq!(driver_16.last_name, "Leclerc");
        assert_eq!(driver_16.tla, "LEC");
        assert_eq!(driver_16.team_name.value, "Ferrari");
    }

    #[test]
    fn test_parse_drivers_invalid() {
        let val = json!("invalid");
        let result = parse_drivers(&val);
        assert!(result.is_err());
    }
}
