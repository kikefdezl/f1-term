use f1_term_core::team::{Team, TeamColor, TeamName};
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use super::Result;

pub fn parse_teams(val: &Value) -> Result<HashMap<TeamName, Team>> {
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
