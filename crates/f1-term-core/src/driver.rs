use super::team::TeamName;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy, Default)]
pub struct DriverNumber {
    pub value: u8,
}

#[derive(Clone, Debug)]
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
