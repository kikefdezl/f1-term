use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawStint {
    pub Compound: String,
    pub LapFlags: u8,
    pub New: String,
    pub StartLaps: u8,
    pub TotalLaps: u8,
    pub TyresNotChanged: String,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawDriverStints {
    pub Stints: Vec<RawStint>,
}

pub fn parse_raw_stints(val: &Value) -> Result<HashMap<String, RawDriverStints>> {
    let mut stints_map: HashMap<String, RawDriverStints> = HashMap::new();
    let lines = val
        .get("Lines")
        .ok_or("Couldn't find 'Lines' in response")?;

    match lines {
        Value::Object(l) => {
            for (num, attrs) in l {
                match RawDriverStints::deserialize(attrs) {
                    Ok(payload) => {
                        stints_map.insert(num.clone(), payload);
                    }
                    Err(e) => {
                        log::warn!("Failed to parse stints payload for driver {}: {}", num, e);
                    }
                }
            }
        }
        _ => return Err("Lines value is not a JSON object".into()),
    }
    Ok(stints_map)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_stints() {
        let json = json!({
            "Lines": {
                "1": {
                    "Stints": [
                        {
                            "Compound": "SOFT",
                            "LapFlags": 0,
                            "New": "true",
                            "StartLaps": 0,
                            "TotalLaps": 15,
                            "TyresNotChanged": "0"
                        },
                        {
                            "Compound": "MEDIUM",
                            "LapFlags": 0,
                            "New": "false",
                            "StartLaps": 3,
                            "TotalLaps": 25,
                            "TyresNotChanged": "0"
                        }
                    ]
                }
            }
        });

        let raw = parse_raw_stints(&json).unwrap();
        assert_eq!(raw.len(), 1);

        let driver_stints = raw.get("1").unwrap();
        assert_eq!(driver_stints.Stints.len(), 2);

        assert_eq!(driver_stints.Stints[0].Compound, "SOFT");
        assert_eq!(driver_stints.Stints[0].New, "true");
        assert_eq!(driver_stints.Stints[0].StartLaps, 0);
        assert_eq!(driver_stints.Stints[0].TotalLaps, 15);

        assert_eq!(driver_stints.Stints[1].Compound, "MEDIUM");
        assert_eq!(driver_stints.Stints[1].New, "false");
        assert_eq!(driver_stints.Stints[1].StartLaps, 3);
        assert_eq!(driver_stints.Stints[1].TotalLaps, 25);
    }

    #[test]
    fn test_stints_missing_fields() {
        let raw_payload = json!({
            "Lines": {
                "44": {
                    "Stints": [
                        {
                            "Compound": "SOFT",
                            "New": "true",
                            "StartLaps": 0,
                            "TotalLaps": 5
                        }
                    ]
                }
            }
        });

        let result = parse_raw_stints(&raw_payload);
        assert!(
            result.is_ok(),
            "Failed to parse stints missing optional fields: {:?}",
            result.err()
        );

        let raw = result.unwrap();
        let driver_stints = raw.get("44").unwrap();

        assert_eq!(driver_stints.Stints.len(), 1);
        assert_eq!(driver_stints.Stints[0].TyresNotChanged, "");
        assert_eq!(driver_stints.Stints[0].LapFlags, 0);
    }
}
