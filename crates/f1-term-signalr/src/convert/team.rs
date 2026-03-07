use std::collections::HashMap;

use f1_term_core::team::{Team, TeamColor, TeamName};
use log::info;

use crate::parsing::driver_list::RawDriver;

impl TryFrom<&RawDriver> for Team {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: &RawDriver) -> Result<Self, Self::Error> {
        Ok(Team {
            name: TeamName {
                value: payload.TeamName.clone(),
            },
            color: TeamColor {
                u32: u32::from_str_radix(&payload.TeamColour, 16)?,
            },
        })
    }
}

pub fn convert_teams(raw_drivers: &HashMap<String, RawDriver>) -> HashMap<TeamName, Team> {
    let mut teams = HashMap::new();

    for (num_str, payload) in raw_drivers {
        match Team::try_from(payload) {
            Ok(team) => {
                teams.insert(team.name.clone(), team);
            }
            Err(e) => {
                info!("Failed to convert team payload for car {}: {}", num_str, e);
            }
        }
    }

    teams
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_teams() {
        let mut raw_drivers = HashMap::new();
        raw_drivers.insert(
            "1".to_string(),
            RawDriver {
                RacingNumber: "1".to_string(),
                FirstName: "Max".to_string(),
                LastName: "Verstappen".to_string(),
                FullName: "Max VERSTAPPEN".to_string(),
                BroadcastName: "M VERSTAPPEN".to_string(),
                HeadshotUrl: "url".to_string(),
                Line: 1,
                PublicIdRight: "id".to_string(),
                Tla: "VER".to_string(),
                TeamName: "Red Bull Racing".to_string(),
                Reference: "ref".to_string(),
                TeamColour: "3671C6".to_string(),
            },
        );

        let teams = convert_teams(&raw_drivers);
        assert_eq!(teams.len(), 1);

        let team = teams
            .get(&TeamName {
                value: "Red Bull Racing".to_string(),
            })
            .unwrap();
        assert_eq!(team.color.u32, 0x3671C6);
    }
}
