pub type Stints = Vec<Stint>;

#[derive(Debug, Clone, PartialEq)]
pub enum Compound {
    Soft,
    Medium,
    Hard,
    Intermediate,
    Wet,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stint {
    pub compound: Compound,
    pub lap_flags: u8,
    pub new: bool,
    pub start_laps: u8,
    pub total_laps: u8,
    pub tires_not_changed: u8,
    pub best_lap: Option<BestLap>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BestLap {
    pub number: u8,
    pub time: String,
}
