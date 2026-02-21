use super::topic::Topic;
use f1_term_core::client::TelemetryEvent;
use f1_term_core::driver::{Driver, DriverNumber};
use f1_term_core::snapshot::FullSnapshot;
use f1_term_core::team::{Team, TeamColor, TeamName};
use f1_term_core::timing::{LastLap, LiveTiming, Sector, Segment, Speed, Speeds};
use log::info;
use serde_json::Value;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_message(text: &str) -> Option<TelemetryEvent> {
    let json: serde_json::Value = serde_json::from_str(text).ok()?;

    let response = json.get("R")?;

    let (drivers, teams) = match response.get(Topic::DriverList.to_string()) {
        None => (HashMap::new(), HashMap::new()),
        Some(dl) => {
            // TODO: If either of these fail right now the whole thing fails, but
            // this shouldn't be and we will need incremental updates
            let drivers: HashMap<DriverNumber, Driver> = parse_drivers(dl).ok()?;
            let teams: HashMap<TeamName, Team> = parse_teams(dl).ok()?;
            (drivers, teams)
        }
    };

    let timing_data = match response.get(Topic::TimingData.to_string()) {
        None => HashMap::new(),
        Some(td) => parse_timing_data(td).unwrap_or_else(|e| {
            info!("Failed to parse timing data: {}", e);
            HashMap::new()
        }),
    };

    let snapshot = FullSnapshot {
        drivers,
        teams,
        timing_data,
    };
    Some(TelemetryEvent::Full(snapshot))
}

fn parse_drivers(val: &Value) -> Result<HashMap<DriverNumber, Driver>> {
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
                match parse_driver(attrs) {
                    Ok(d) => {
                        drivers.insert(driver_number, d);
                    }
                    Err(e) => {
                        info!("Failed to parse driver with attrs {}: {}", attrs, e);
                    }
                }
            }
        }
        _ => return Err("Drivers value is not a JSON object".into()),
    }
    Ok(drivers)
}

fn parse_driver(val: &Value) -> Result<Driver> {
    match val {
        Value::Object(attrs) => Ok(Driver {
            number: DriverNumber {
                value: attrs
                    .get("RacingNumber")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid RacingNumber")?
                    .parse()?,
            },
            first_name: attrs
                .get("FirstName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid FirstName")?
                .to_string(),
            last_name: attrs
                .get("LastName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid LastName")?
                .to_string(),
            full_name: attrs
                .get("FullName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid FullName")?
                .to_string(),
            broadcast_name: attrs
                .get("BroadcastName")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid BroadcastName")?
                .to_string(),
            headshot_url: attrs
                .get("HeadshotUrl")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid HeadshotUrl")?
                .to_string(),
            line: attrs.get("Line").and_then(|v| v.as_u64()).map(|v| v as u8),
            public_id_right: attrs
                .get("PublicIdRight")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid PublicIdRight")?
                .to_string(),
            tla: attrs
                .get("Tla")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid Tla")?
                .to_string(),
            team_name: TeamName {
                value: attrs
                    .get("TeamName")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid TeamName")?
                    .to_string(),
            },
            reference: attrs
                .get("Reference")
                .and_then(|v| v.as_str())
                .ok_or("Missing or invalid Reference")?
                .to_string(),
        }),
        _ => Err("Error parsing driver: attrs is not an Object".into()),
    }
}

fn parse_teams(val: &Value) -> Result<HashMap<TeamName, Team>> {
    let mut teams: HashMap<TeamName, Team> = HashMap::new();
    match val {
        Value::Object(map) => {
            for (_, attrs) in map.iter() {
                // Medical and safety cars don't have a team, so those fail to parse.
                // We just ignore them.
                match parse_team(attrs) {
                    Ok(t) => {
                        teams.insert(t.name.clone(), t);
                    }
                    Err(e) => {
                        info!("Failed to parse team with attrs {}: {}", attrs, e);
                    }
                }
            }
        }
        _ => return Err("Drivers value is not a JSON object".into()),
    }
    Ok(teams)
}

fn parse_team(val: &Value) -> Result<Team> {
    match val {
        Value::Object(attrs) => Ok(Team {
            name: TeamName {
                value: attrs
                    .get("TeamName")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing or invalid TeamName")?
                    .to_string(),
            },
            color: TeamColor {
                u32: attrs
                    .get("TeamColour")
                    .and_then(|v| v.as_str())
                    .and_then(|v| u32::from_str_radix(v, 16).ok())
                    .ok_or("Missing or invalid TeamColour")?,
            },
        }),
        _ => Err("Error parsing team from driver, should be a JSON Object".into()),
    }
}

fn parse_timing_data(val: &Value) -> Result<HashMap<DriverNumber, LiveTiming>> {
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
                    .unwrap_or("")
                    .to_string(),
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
            .unwrap_or("")
            .to_string(),
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
