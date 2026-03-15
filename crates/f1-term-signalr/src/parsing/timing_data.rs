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
    pub TimeDiffToPositionAhead: String,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RawIntervalToPositionAhead {
    pub Catching: bool,
    pub Value: String,
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
    pub TimeDiffToFastest: Option<String>,
    pub TimeDiffToPositionAhead: Option<String>,
    pub Sectors: Vec<RawSector>,
    pub ShowPosition: bool,
    pub Speeds: RawSpeeds,
    pub Cutoff: Option<bool>,
    pub KnockedOut: Option<bool>,
    pub NumberOfLaps: Option<u8>,
    pub NumberOfPitStops: Option<u8>,
    pub Stats: Option<Vec<RawStats>>,
    pub GapToLeader: Option<String>,
    pub IntervalToPositionAhead: Option<RawIntervalToPositionAhead>,
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
    use serde_json::json;

    use super::*;

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
        assert_eq!(raw.len(), 1);

        let timing = raw.get("1").unwrap();

        assert_eq!(timing.Position, "1");
        assert_eq!(timing.BestLapTime.Value, "1:23.456");
        assert_eq!(timing.LastLapTime.Value, "1:24.000");
        assert!(timing.LastLapTime.PersonalFastest);

        assert_eq!(timing.Sectors.len(), 1);
        assert_eq!(timing.Sectors[0].Value, "25.1");
        assert!(timing.Sectors[0].PersonalFastest);

        assert_eq!(timing.Speeds.FL.Value, "320");
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
        let driver_timing = raw.get("44").unwrap();

        assert_eq!(driver_timing.BestLapTime.Value, "");
        assert_eq!(driver_timing.LastLapTime.Value, "");
        assert_eq!(driver_timing.TimeDiffToFastest, None);
    }

    #[test]
    fn test_parse_timing_data_quali_stats() {
        let json = json!({
            "Lines": {
                "1": {
                    "RacingNumber": "1",
                    "Stats": [
                        {
                            "TimeDiffToFastest": "+0.123",
                            "TimeDiffToPositionAhead": "+0.050"
                        }
                    ],
                    "BestLapTime": {
                        "Value": "1:23.456"
                    },
                    "LastLapTime": {
                        "OverallFastest": false,
                        "PersonalFastest": true,
                        "Status": 0,
                        "Value": "1:24.000"
                    },
                    "Position": "1",
                    "Status": 0,
                    "Stopped": false,
                    "ShowPosition": true,
                    "Sectors": [],
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
        let timing = raw.get("1").unwrap();

        let stats = timing.Stats.as_ref().unwrap();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].TimeDiffToFastest, "+0.123");
        assert_eq!(stats[0].TimeDiffToPositionAhead, "+0.050");
    }
}
