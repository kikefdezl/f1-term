use super::topic::Topic;
use f1_term_core::client::TelemetryEvent;
use f1_term_core::driver::{Driver, DriverNumber};
use f1_term_core::snapshot::FullSnapshot;
use f1_term_core::team::{Team, TeamColor, TeamName};
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
    let snapshot = FullSnapshot { drivers, teams };
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
