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

impl Stint {
    pub fn laps_done(&self) -> u8 {
        self.total_laps - self.start_laps
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn stint() -> Stint {
        Stint {
            compound: Compound::Medium,
            lap_flags: 0,
            new: false,
            start_laps: 8,
            total_laps: 14,
            tires_not_changed: 0,
            best_lap: Some(BestLap {
                number: 23,
                time: "1:23.456".to_string(),
            }),
        }
    }

    #[test]
    fn test_laps_done() {
        let stint = stint();
        assert_eq!(stint.laps_done(), 6);
    }
}
