use std::collections::HashMap;

use log::info;
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct RawDriver {
    pub RacingNumber: String,
    pub FirstName: String,
    pub LastName: String,
    pub FullName: String,
    pub BroadcastName: String,
    pub HeadshotUrl: String,
    pub Line: u8,
    pub PublicIdRight: String,
    pub Tla: String,
    pub TeamName: String,
    pub Reference: String,
    pub TeamColour: String,
}

pub fn parse_driver_list(val: &Value) -> Result<HashMap<String, RawDriver>> {
    let mut raw_drivers: HashMap<String, RawDriver> = HashMap::new();
    match val {
        Value::Object(map) => {
            for (num_str, attrs) in map.iter() {
                if num_str == "_kf" {
                    continue;
                }

                match RawDriver::deserialize(attrs) {
                    Ok(payload) => {
                        raw_drivers.insert(num_str.clone(), payload);
                    }
                    Err(e) => {
                        // Medical and safety cars don't have all fields, so those fail to parse.
                        // We just ignore them.
                        info!("Failed to parse driver payload for {}: {}", num_str, e);
                    }
                }
            }
        }
        _ => return Err("Drivers value is not a JSON object".into()),
    }
    Ok(raw_drivers)
}

#[cfg(test)]
mod tests {
    use f1_term_core::{driver::DriverNumber, team::TeamName};
    use serde_json::json;

    use super::*;
    use crate::convert::{driver::convert_drivers, team::convert_teams};

    fn driver_1() -> serde_json::Value {
        json!({
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
            "Reference": "MAXVER01",
            "TeamColour": "3671C6"
        })
    }

    fn driver_2() -> serde_json::Value {
        json!({
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
            "Reference": "CHALEC01",
            "TeamColour": "F91536"
        })
    }

    #[test]
    fn test_parse_drivers() {
        let val = json!({
            "1": driver_1(),
            "16": driver_2(),
            "SC": {
                "RacingNumber": "SC",
                "SomeOtherField": "Medical Car"
            }
        });

        let raw_drivers = parse_driver_list(&val).unwrap();
        let drivers = convert_drivers(&raw_drivers);

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
        let result = parse_driver_list(&val);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_teams() {
        let json = json!({
            "1": driver_1(),
            "16": driver_2(),
            "invalid": {
                "NotATeam": "Something"
            }
        });

        let raw_drivers = parse_driver_list(&json).unwrap();
        let teams = convert_teams(&raw_drivers);
        assert_eq!(teams.len(), 2);

        let rb = teams
            .get(&TeamName {
                value: "Red Bull Racing".to_string(),
            })
            .unwrap();
        assert_eq!(rb.color.u32, 0x3671C6);

        let ferrari = teams
            .get(&TeamName {
                value: "Ferrari".to_string(),
            })
            .unwrap();
        assert_eq!(ferrari.color.u32, 0xF91536);
    }
}
