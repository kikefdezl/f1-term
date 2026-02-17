use std::fmt::Display;

use super::team::TeamName;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct DriverNumber {
    pub value: u8,
}

#[derive(Debug)]
pub struct Driver {
    pub number: DriverNumber,
    pub team_name: TeamName,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub broadcast_name: String,
    pub headshot_url: String,
    pub line: Option<u8>,
    pub public_id_right: String,
    /// Three Letter Acronym
    pub tla: String,
    pub reference: String,
}

impl Display for Driver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.tla)?;
        writeln!(f, "{}", self.full_name)?;
        writeln!(f, "Number: {}", self.number.value)?;
        writeln!(f, "Team: {}", self.team_name.value)
    }
}
