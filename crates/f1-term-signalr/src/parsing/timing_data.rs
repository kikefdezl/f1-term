use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawBestLapTime {
    pub Value: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawLastLapTime {
    pub OverallFastest: bool,
    pub PersonalFastest: bool,
    pub Status: u32,
    pub Value: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawSegment {
    pub Status: u32,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawSector {
    pub OverallFastest: bool,
    pub PersonalFastest: bool,
    pub Segments: Vec<RawSegment>,
    pub Status: u32,
    pub Stopped: bool,
    pub Value: String,
    pub PreviousValue: Option<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawSpeed {
    pub OverallFastest: bool,
    pub PersonalFastest: bool,
    pub Status: u32,
    pub Value: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawSpeeds {
    pub FL: RawSpeed,
    pub I1: RawSpeed,
    pub I2: RawSpeed,
    pub ST: RawSpeed,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawStats {
    pub TimeDiffToFastest: String,
    pub TimeDifftoPositionAhead: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawTimingData {
    pub RacingNumber: String,
    pub BestLapTime: RawBestLapTime,
    pub InPit: bool,
    pub PitOut: bool,
    pub LastLapTime: RawLastLapTime,
    pub Position: String,
    pub Retired: bool,
    pub Status: u32,
    pub Stopped: bool,
    pub TimeDiffToFastest: String,
    pub TimeDiffToPositionAhead: String,
    pub Sectors: Vec<RawSector>,
    pub ShowPosition: bool,
    pub Speeds: RawSpeeds,
    pub Cutoff: Option<bool>,
    pub KnockedOut: Option<bool>,
    pub NumberOfLaps: Option<u8>,
    pub NumberOfPitStops: Option<u8>,
    pub Stats: Option<Vec<RawStats>>,
}

pub fn parse_raw_timing_data(val: &Value) -> Result<HashMap<String, RawTimingData>> {
    let mut timing_data: HashMap<String, RawTimingData> = HashMap::new();
    let lines = val.get("Lines").ok_or("Missing Lines in TimingData")?;

    match lines {
        Value::Object(map) => {
            for (num, attrs) in map.iter() {
                match RawTimingData::deserialize(attrs) {
                    Ok(payload) => {
                        timing_data.insert(num.clone(), payload);
                    }
                    Err(e) => {
                        log::info!("Failed to parse timing data payload for {}: {}", num, e);
                    }
                }
            }
        }
        _ => return Err("TimingData Lines is not an object".into()),
    }
    Ok(timing_data)
}

#[cfg(test)]
mod tests {
    use f1_term_core::driver::DriverNumber;
    use serde_json::json;

    use super::*;
    use crate::convert::timing::convert_timing_data;

    #[test]
    fn test_parse_timing_data() {
        let json = json!({
            "Lines": {
                "1": {
                    "RacingNumber": "1",
                    "BestLapTime": {
                        "Value": "1:23.456"
                    },
                    "InPit": false,
                    "PitOut": false,
                    "LastLapTime": {
                        "OverallFastest": false,
                        "PersonalFastest": true,
                        "Status": 0,
                        "Value": "1:24.000"
                    },
                    "Position": "1",
                    "Retired": false,
                    "Status": 0,
                    "Stopped": false,
                    "TimeDiffToFastest": "",
                    "TimeDiffToPositionAhead": "",
                    "Sectors": [
                        {
                            "OverallFastest": false,
                            "PersonalFastest": true,
                            "Segments": [
                                { "Status": 0 }
                            ],
                            "Status": 0,
                            "Stopped": false,
                            "Value": "25.1",
                            "PreviousValue": "25.6"
                        }
                    ],
                    "ShowPosition": true,
                    "Speeds": {
                        "FL": { "OverallFastest": false, "PersonalFastest": false, "Status": 0, "Value": "320" },
                        "I1": { "OverallFastest": false, "PersonalFastest": false, "Status": 0, "Value": "" },
                        "I2": { "OverallFastest": false, "PersonalFastest": false, "Status": 0, "Value": "" },
                        "ST": { "OverallFastest": false, "PersonalFastest": false, "Status": 0, "Value": "" }
                    }
                }
            }
        });

        let raw = parse_raw_timing_data(&json).unwrap();
        let data = convert_timing_data(&raw);
        assert_eq!(data.len(), 1);

        let driver_number = DriverNumber { value: 1 };
        let timing = data.get(&driver_number).unwrap();

        assert_eq!(timing.position, 1);
        assert_eq!(timing.lap_data.best_lap_time.as_deref(), Some("1:23.456"));
        assert_eq!(timing.lap_data.last_lap.time.as_deref(), Some("1:24.000"));
        assert!(timing.lap_data.last_lap.personal_fastest);

        assert_eq!(timing.lap_data.last_lap.sectors.len(), 1);
        assert_eq!(
            timing.lap_data.last_lap.sectors[0].value.as_deref(),
            Some("25.1")
        );
        assert!(timing.lap_data.last_lap.sectors[0].personal_fastest);

        assert_eq!(timing.lap_data.last_lap.speeds.fl.value, "320");
    }

    #[test]
    fn test_timing_data_missing_fields() {
        let raw_payload = json!({
            "Lines": {
                "44": {
                    "RacingNumber": "44",
                    "Position": "1",
                    "InPit": false,
                    "PitOut": false,
                    "Retired": false,
                    "Status": 0,
                    "Stopped": false,
                    "ShowPosition": true,
                    "Sectors": [],
                    "Speeds": {
                        "FL": { "Status": 0 },
                        "I1": { "Status": 0 },
                        "I2": { "Status": 0 },
                        "ST": { "Status": 0 }
                    }
                }
            }
        });

        let result = parse_raw_timing_data(&raw_payload);
        assert!(
            result.is_ok(),
            "Failed to parse payload missing optional fields: {:?}",
            result.err()
        );

        let raw = result.unwrap();
        let map = convert_timing_data(&raw);
        let driver_timing = map.get(&DriverNumber { value: 44 }).unwrap();

        assert_eq!(driver_timing.lap_data.best_lap_time, None);
        assert_eq!(driver_timing.lap_data.last_lap.time, None);
        assert_eq!(driver_timing.time_diffs.to_fastest, None);
    }
}
