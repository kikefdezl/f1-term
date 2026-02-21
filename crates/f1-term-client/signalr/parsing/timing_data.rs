use f1_term_core::timing::{LastLap, LiveTiming, Sector, Segment, Speed, Speeds};

use super::Result;
use f1_term_core::driver::DriverNumber;
use log::info;
use serde_json::Value;
use std::collections::HashMap;

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
                match parse_live_timing(driver_number, attrs) {
                    Ok(lt) => {
                        timing_data.insert(driver_number, lt);
                    }
                    Err(e) => {
                        info!("Failed to parse live timing for {}: {}", num, e);
                    }
                }
            }
        }
        _ => return Err("TimingData Lines is not an object".into()),
    }
    Ok(timing_data)
}

fn parse_live_timing(driver_number: DriverNumber, val: &Value) -> Result<LiveTiming> {
    match val {
        Value::Object(attrs) => {
            let last_lap_time = attrs.get("LastLapTime").ok_or("Missing LastLapTime")?;

            Ok(LiveTiming {
                driver_number,
                best_lap_time: attrs
                    .get("BestLapTime")
                    .and_then(|v| v.get("Value"))
                    .and_then(|v| v.as_str())
                    .filter(|v| !v.is_empty())
                    .map(|v| v.to_string()),
                in_pit: attrs
                    .get("InPit")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                pit_out: attrs
                    .get("PitOut")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                last_lap: parse_last_lap(last_lap_time, attrs)?,
                position: attrs
                    .get("Position")
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                retired: attrs
                    .get("Retired")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                status: attrs
                    .get("Status")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u8)
                    .unwrap_or(0),
                stopped: attrs
                    .get("Stopped")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                time_diff_to_fastest: attrs
                    .get("TimeDiffToFastest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                time_diff_to_position_ahead: attrs
                    .get("TimeDiffToPositionAhead")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
        }
        _ => Err("Live timing value is not an object".into()),
    }
}

fn parse_last_lap(last_lap_time: &Value, root: &serde_json::Map<String, Value>) -> Result<LastLap> {
    Ok(LastLap {
        overall_fastest: last_lap_time
            .get("OverallFastest")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        personal_fastest: last_lap_time
            .get("PersonalFastest")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        status: last_lap_time
            .get("Status")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
            .unwrap_or(0),
        time: last_lap_time
            .get("Value")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        sectors: parse_sectors(root.get("Sectors"))?,
        show_position: root
            .get("ShowPosition")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        speeds: parse_speeds(root.get("Speeds"))?,
    })
}

fn parse_sectors(val: Option<&Value>) -> Result<Vec<Sector>> {
    match val {
        Some(Value::Array(arr)) => {
            let mut sectors = Vec::new();
            for s in arr {
                sectors.push(parse_sector(s)?);
            }
            Ok(sectors)
        }
        _ => Ok(Vec::new()), // Empty sectors if missing or not array
    }
}

fn parse_sector(val: &Value) -> Result<Sector> {
    match val {
        Value::Object(attrs) => Ok(Sector {
            overall_fastest: attrs
                .get("OverallFastest")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            personal_fastest: attrs
                .get("PersonalFastest")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            segments: parse_segments(attrs.get("Segments"))?,
            status: attrs
                .get("Status")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .unwrap_or(0),
            stopped: attrs
                .get("Stopped")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            value: attrs
                .get("Value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        _ => Err("Sector is not an object".into()),
    }
}

fn parse_segments(val: Option<&Value>) -> Result<Vec<Segment>> {
    match val {
        Some(Value::Array(arr)) => {
            let mut segments = Vec::new();
            for s in arr {
                segments.push(parse_segment(s)?);
            }
            Ok(segments)
        }
        _ => Ok(Vec::new()),
    }
}

fn parse_segment(val: &Value) -> Result<Segment> {
    match val {
        Value::Object(attrs) => Ok(Segment {
            status: attrs
                .get("Status")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .unwrap_or(0),
        }),
        _ => Err("Segment is not an object".into()),
    }
}

fn parse_speeds(val: Option<&Value>) -> Result<Speeds> {
    let empty_speed = Speed {
        overall_fastest: false,
        personal_fastest: false,
        status: 0,
        value: "".to_string(),
    };

    match val {
        Some(Value::Object(attrs)) => Ok(Speeds {
            fl: attrs
                .get("FL")
                .map(parse_speed)
                .unwrap_or(Ok(empty_speed.clone()))?,
            i1: attrs
                .get("I1")
                .map(parse_speed)
                .unwrap_or(Ok(empty_speed.clone()))?,
            i2: attrs
                .get("I2")
                .map(parse_speed)
                .unwrap_or(Ok(empty_speed.clone()))?,
            st: attrs
                .get("ST")
                .map(parse_speed)
                .unwrap_or(Ok(empty_speed))?,
        }),
        _ => Ok(Speeds {
            fl: empty_speed.clone(),
            i1: empty_speed.clone(),
            i2: empty_speed.clone(),
            st: empty_speed,
        }),
    }
}

fn parse_speed(val: &Value) -> Result<Speed> {
    match val {
        Value::Object(attrs) => Ok(Speed {
            overall_fastest: attrs
                .get("OverallFastest")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            personal_fastest: attrs
                .get("PersonalFastest")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            status: attrs
                .get("Status")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .unwrap_or(0),
            value: attrs
                .get("Value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        _ => Err("Speed is not an object".into()),
    }
}
