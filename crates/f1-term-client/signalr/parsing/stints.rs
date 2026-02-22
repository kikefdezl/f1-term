use f1_term_core::driver::DriverNumber;
use f1_term_core::stint::{Compound, Stint, Stints};
use log::info;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct StintPayload {
    Compound: String,
    LapFlags: u8,
    New: String,
    StartLaps: u8,
    TotalLaps: u8,
    TyresNotChanged: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct DriverStintsPayload {
    Stints: Vec<StintPayload>,
}

impl TryFrom<StintPayload> for Stint {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: StintPayload) -> Result<Self> {
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
            tires_not_changed: payload.TyresNotChanged.parse().expect("Should be a number"),
        })
    }
}

pub fn parse_stints(val: &Value) -> Result<HashMap<DriverNumber, Stints>> {
    let mut stints_map: HashMap<DriverNumber, Stints> = HashMap::new();
    let lines = val
        .get("Lines")
        .ok_or("Couldn't find 'Lines' in response")?;

    match lines {
        Value::Object(l) => {
            for (num, attrs) in l {
                let number: u8 = match num.parse() {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                let driver_number = DriverNumber { value: number };

                match serde_json::from_value::<DriverStintsPayload>(attrs.clone()) {
                    Ok(payload) => {
                        let driver_stints: Stints = payload
                            .Stints
                            .into_iter()
                            .filter_map(|s| match Stint::try_from(s) {
                                Ok(stint) => Some(stint),
                                Err(e) => {
                                    info!("Failed to parse stint for driver {}: {}", num, e);
                                    None
                                }
                            })
                            .collect();
                        stints_map.insert(driver_number, driver_stints);
                    }
                    Err(e) => {
                        info!("Failed to parse stints payload for driver {}: {}", num, e);
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
    use super::*;
    use serde_json::json;

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
                            "TyresNotChanged": 0
                        },
                        {
                            "Compound": "MEDIUM",
                            "LapFlags": 0,
                            "New": "false",
                            "StartLaps": 3,
                            "TotalLaps": 25,
                            "TyresNotChanged": 0
                        }
                    ]
                }
            }
        });

        let stints_map = parse_stints(&json).unwrap();
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
}
