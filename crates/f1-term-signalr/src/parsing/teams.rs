use std::collections::HashMap;

use f1_term_core::team::{Team, TeamColor, TeamName};
use log::info;
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct TeamPayload {
    TeamName: String,
    TeamColour: String,
}

impl TryFrom<TeamPayload> for Team {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: TeamPayload) -> Result<Self> {
        Ok(Team {
            name: TeamName {
                value: payload.TeamName,
            },
            color: TeamColor {
                u32: u32::from_str_radix(&payload.TeamColour, 16)?,
            },
        })
    }
}

pub fn parse_teams(val: &Value) -> Result<HashMap<TeamName, Team>> {
    let mut teams: HashMap<TeamName, Team> = HashMap::new();
    match val {
        Value::Object(map) => {
            for (_, attrs) in map.iter() {
                // Medical and safety cars don't have a team, so those fail to parse.
                // We just ignore them.
                match TeamPayload::deserialize(attrs) {
                    Ok(payload) => match Team::try_from(payload) {
                        Ok(t) => {
                            teams.insert(t.name.clone(), t);
                        }
                        Err(e) => {
                            info!("Failed to convert team payload: {}", e);
                        }
                    },
                    Err(e) => {
                        info!("Failed to parse team payload: {}", e);
                    }
                }
            }
        }
        _ => return Err("Drivers value is not a JSON object".into()),
    }
    Ok(teams)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_teams() {
        let json = json!({
            "1": {
                "TeamName": "Red Bull Racing",
                "TeamColour": "3671C6"
            },
            "2": {
                "TeamName": "Ferrari",
                "TeamColour": "F91536"
            },
            "invalid": {
                "NotATeam": "Something"
            }
        });

        let teams = parse_teams(&json).unwrap();
        assert_eq!(teams.len(), 2);

        let rb = teams
            .get(&TeamName {
                value: "Red Bull Racing".to_string(),
            })
            .unwrap();
        assert_eq!(rb.color.u32, 0x3671C6);

        let ferrari = teams
            .get(&TeamName {
                value: "Ferrari".to_string(),
            })
            .unwrap();
        assert_eq!(ferrari.color.u32, 0xF91536);
    }
}
