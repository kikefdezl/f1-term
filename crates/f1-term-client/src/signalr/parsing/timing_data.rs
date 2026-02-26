use std::collections::HashMap;

use f1_term_core::{
    driver::DriverNumber,
    timing::{LastLap, LiveTiming, Sector, Segment, SegmentStatus, Speed, Speeds},
};
use log::{info, warn};
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
struct BestLapTimePayload {
    Value: String,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
struct LastLapTimePayload {
    OverallFastest: bool,
    PersonalFastest: bool,
    Status: u32,
    Value: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct SegmentPayload {
    Status: u32,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct SectorPayload {
    OverallFastest: bool,
    PersonalFastest: bool,
    Segments: Vec<SegmentPayload>,
    Status: u32,
    Stopped: bool,
    Value: String,
    PreviousValue: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct SpeedPayload {
    OverallFastest: bool,
    PersonalFastest: bool,
    Status: u32,
    Value: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct SpeedsPayload {
    FL: SpeedPayload,
    I1: SpeedPayload,
    I2: SpeedPayload,
    ST: SpeedPayload,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct LiveTimingPayload {
    RacingNumber: String,
    BestLapTime: BestLapTimePayload,
    InPit: bool,
    PitOut: bool,
    LastLapTime: LastLapTimePayload,
    Position: String,
    Retired: bool,
    Status: u32,
    Stopped: bool,
    TimeDiffToFastest: String,
    TimeDiffToPositionAhead: String,
    Sectors: Vec<SectorPayload>,
    ShowPosition: bool,
    Speeds: SpeedsPayload,
}

impl From<SegmentPayload> for Segment {
    fn from(p: SegmentPayload) -> Self {
        let status = match p.Status {
            2048 => SegmentStatus::Normal,
            2049 => SegmentStatus::PersonalFastest,
            2064 => SegmentStatus::InPit,
            other => {
                warn!("Unknown SegmentStatus value {}!", other);
                SegmentStatus::None
            }
        };
        Segment { status }
    }
}

impl From<SectorPayload> for Sector {
    fn from(p: SectorPayload) -> Self {
        let value = Some(p.Value).filter(|s| !s.is_empty());
        Sector {
            overall_fastest: p.OverallFastest,
            personal_fastest: p.PersonalFastest,
            segments: p.Segments.into_iter().map(Into::into).collect(),
            status: p.Status,
            stopped: p.Stopped,
            value,
            previous_value: p.PreviousValue,
        }
    }
}

impl From<SpeedPayload> for Speed {
    fn from(p: SpeedPayload) -> Self {
        Speed {
            overall_fastest: p.OverallFastest,
            personal_fastest: p.PersonalFastest,
            status: p.Status,
            value: p.Value,
        }
    }
}

impl From<SpeedsPayload> for Speeds {
    fn from(p: SpeedsPayload) -> Self {
        Speeds {
            fl: p.FL.into(),
            i1: p.I1.into(),
            i2: p.I2.into(),
            st: p.ST.into(),
        }
    }
}

impl TryFrom<LiveTimingPayload> for LiveTiming {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: LiveTimingPayload) -> Result<Self> {
        let driver_number = DriverNumber {
            value: payload.RacingNumber.parse()?,
        };

        // API returns empty strings, we convert those to None
        let best_lap_time = Some(payload.BestLapTime.Value).filter(|s| !s.is_empty());
        let last_lap_time = Some(payload.LastLapTime.Value).filter(|s| !s.is_empty());
        let time_diff_to_fastest = Some(payload.TimeDiffToFastest).filter(|s| !s.is_empty());
        let time_diff_to_position_ahead =
            Some(payload.TimeDiffToPositionAhead).filter(|s| !s.is_empty());

        let last_lap = LastLap {
            overall_fastest: payload.LastLapTime.OverallFastest,
            personal_fastest: payload.LastLapTime.PersonalFastest,
            status: payload.LastLapTime.Status,
            time: last_lap_time,
            sectors: payload.Sectors.into_iter().map(Into::into).collect(),
            show_position: payload.ShowPosition,
            speeds: payload.Speeds.into(),
        };

        Ok(LiveTiming {
            driver_number,
            best_lap_time,
            in_pit: payload.InPit,
            pit_out: payload.PitOut,
            last_lap,
            position: payload.Position.parse().unwrap_or(0),
            retired: payload.Retired,
            status: payload.Status,
            stopped: payload.Stopped,
            time_diff_to_fastest,
            time_diff_to_position_ahead,
        })
    }
}

pub fn parse_timing_data(val: &Value) -> Result<HashMap<DriverNumber, LiveTiming>> {
    let mut timing_data: HashMap<DriverNumber, LiveTiming> = HashMap::new();
    let lines = val.get("Lines").ok_or("Missing Lines in TimingData")?;

    match lines {
        Value::Object(map) => {
            for (num, attrs) in map.iter() {
                let number: u8 = match num.parse() {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                let driver_number = DriverNumber { value: number };
                match serde_json::from_value::<LiveTimingPayload>(attrs.clone()) {
                    Ok(payload) => match LiveTiming::try_from(payload) {
                        Ok(lt) => {
                            timing_data.insert(driver_number, lt);
                        }
                        Err(e) => {
                            info!("Failed to convert live timing payload for {}: {}", num, e);
                        }
                    },
                    Err(e) => {
                        info!("Failed to parse live timing payload for {}: {}", num, e);
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

        let data = parse_timing_data(&json).unwrap();
        assert_eq!(data.len(), 1);

        let driver_number = DriverNumber { value: 1 };
        let timing = data.get(&driver_number).unwrap();

        assert_eq!(timing.position, 1);
        assert_eq!(timing.best_lap_time.as_deref(), Some("1:23.456"));
        assert_eq!(timing.last_lap.time.as_deref(), Some("1:24.000"));
        assert!(timing.last_lap.personal_fastest);

        assert_eq!(timing.last_lap.sectors.len(), 1);
        assert_eq!(timing.last_lap.sectors[0].value, "25.1");
        assert!(timing.last_lap.sectors[0].personal_fastest);

        assert_eq!(timing.last_lap.speeds.fl.value, "320");
    }
}
