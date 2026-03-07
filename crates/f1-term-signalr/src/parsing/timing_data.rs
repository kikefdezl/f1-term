use std::collections::HashMap;

use f1_term_core::{
    driver::DriverNumber,
    timing::{LastLap, LiveTiming, Sector, Segment, SegmentStatus, Speed, Speeds, TimeDiffs},
};
use log::{info, warn};
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct BestLapTimePayload {
    Value: String,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct LastLapTimePayload {
    OverallFastest: bool,
    PersonalFastest: bool,
    Status: u32,
    Value: String,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct SegmentPayload {
    Status: u32,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct SectorPayload {
    OverallFastest: bool,
    PersonalFastest: bool,
    Segments: Vec<SegmentPayload>,
    Status: u32,
    Stopped: bool,
    Value: String,
    PreviousValue: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct SpeedPayload {
    OverallFastest: bool,
    PersonalFastest: bool,
    Status: u32,
    Value: String,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct SpeedsPayload {
    FL: SpeedPayload,
    I1: SpeedPayload,
    I2: SpeedPayload,
    ST: SpeedPayload,
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct StatsPayload {
    TimeDiffToFastest: String,
    TimeDifftoPositionAhead: String,
}

impl From<StatsPayload> for TimeDiffs {
    fn from(value: StatsPayload) -> Self {
        let to_fastest = Some(value.TimeDiffToFastest).filter(|s| !s.is_empty());
        let to_position_ahead = Some(value.TimeDifftoPositionAhead).filter(|s| !s.is_empty());
        Self {
            to_fastest,
            to_position_ahead,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
#[serde(default)]
struct TimingDataPayload {
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
    Cutoff: Option<bool>,
    KnockedOut: Option<bool>,
    NumberOfLaps: Option<u8>,
    NumberOfPitStops: Option<u8>,
    Stats: Option<Vec<StatsPayload>>,
}

impl From<SegmentPayload> for Segment {
    fn from(p: SegmentPayload) -> Self {
        let status = match p.Status {
            0 => SegmentStatus::None,
            2048 => SegmentStatus::Normal,
            2049 => SegmentStatus::PersonalFastest,
            2050 => SegmentStatus::Unknown,
            2051 => SegmentStatus::OverallFastest,
            2052 => SegmentStatus::Aborted,
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

impl TryFrom<TimingDataPayload> for LiveTiming {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: TimingDataPayload) -> Result<Self> {
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

        let time_diffs = TimeDiffs {
            to_fastest: time_diff_to_fastest,
            to_position_ahead: time_diff_to_position_ahead,
        };

        let quali_stats = payload.Stats.map(|s| {
            s.into_iter()
                .map(|stat| stat.into())
                .collect::<Vec<TimeDiffs>>()
        });
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
            time_diffs,
            cutoff: payload.Cutoff,
            knocked_out: payload.KnockedOut,
            number_of_laps: payload.NumberOfLaps,
            number_of_pit_stops: payload.NumberOfPitStops,
            quali_stats,
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
                    Err(_) => {
                        warn!("Failed to parse timing data line {num}");
                        continue;
                    }
                };
                let driver_number = DriverNumber { value: number };
                match TimingDataPayload::deserialize(attrs) {
                    Ok(payload) => match LiveTiming::try_from(payload) {
                        Ok(lt) => {
                            timing_data.insert(driver_number, lt);
                        }
                        Err(e) => {
                            info!("Failed to convert live timing payload for {}: {}", num, e);
                        }
                    },
                    Err(e) => {
                        info!("Failed to parse timing data payload for {}: {}", num, e);
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
        assert_eq!(timing.last_lap.sectors[0].value.as_deref(), Some("25.1"));
        assert!(timing.last_lap.sectors[0].personal_fastest);

        assert_eq!(timing.last_lap.speeds.fl.value, "320");
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

        let result = parse_timing_data(&raw_payload);
        assert!(
            result.is_ok(),
            "Failed to parse payload missing optional fields: {:?}",
            result.err()
        );

        let map = result.unwrap();
        let driver_timing = map.get(&DriverNumber { value: 44 }).unwrap();

        assert_eq!(driver_timing.best_lap_time, None);
        assert_eq!(driver_timing.last_lap.time, None);
        assert_eq!(driver_timing.time_diffs.to_fastest, None);
    }
}
