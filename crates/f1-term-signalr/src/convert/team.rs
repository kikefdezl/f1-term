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
