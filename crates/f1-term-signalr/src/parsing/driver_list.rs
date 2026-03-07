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
    use serde_json::json;

    use super::*;

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

        assert_eq!(raw_drivers.len(), 2);

        let driver_1 = raw_drivers.get("1").unwrap();
        assert_eq!(driver_1.FirstName, "Max");
        assert_eq!(driver_1.LastName, "Verstappen");
        assert_eq!(driver_1.Tla, "VER");
        assert_eq!(driver_1.TeamName, "Red Bull Racing");

        let driver_16 = raw_drivers.get("16").unwrap();
        assert_eq!(driver_16.FirstName, "Charles");
        assert_eq!(driver_16.LastName, "Leclerc");
        assert_eq!(driver_16.Tla, "LEC");
        assert_eq!(driver_16.TeamName, "Ferrari");
    }

    #[test]
    fn test_parse_drivers_invalid() {
        let val = json!("invalid");
        let result = parse_driver_list(&val);
        assert!(result.is_err());
    }
}
