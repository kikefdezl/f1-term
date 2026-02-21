pub type Stints = Vec<Stint>;

#[derive(Debug)]
pub enum Compound {
    Soft,
    Medium,
    Hard,
    Inter,
    Wet,
}

#[derive(Debug)]
pub struct Stint {
    pub compound: Compound,
    pub lap_flags: u8,
    pub new: bool,
    pub start_laps: u8,
    pub total_laps: u8,
    pub tires_not_changed: u8,
}
