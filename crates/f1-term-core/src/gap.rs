use std::cmp::Ordering;
use std::fmt::Display;

use crate::lap_time::LapTime;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Gap {
    Time(LapTime),
    Laps(u8),
}

impl TryFrom<&str> for Gap {
    type Error = Box<dyn std::error::Error>;

    /// Parses the enum from a string of type:
    /// - Time: "1:23.345"
    /// - Laps: "+1L"
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("Empty input value".into());
        }
        if value.starts_with("+") && value.ends_with("L") {
            if let Ok(laps) = value[1..value.len() - 1].parse() {
                Ok(Self::Laps(laps))
            } else {
                Err("Failed to parse laps".into())
            }
        } else {
            let time = LapTime::try_from(value)?;
            Ok(Self::Time(time))
        }
    }
}

impl Ord for Gap {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Gap::Laps(laps_self) => match other {
                Gap::Time(_) => Ordering::Greater,
                Gap::Laps(laps_other) => laps_self.cmp(laps_other),
            },
            Gap::Time(time_self) => match other {
                Gap::Laps(_) => Ordering::Less,
                Gap::Time(time_other) => time_self.cmp(time_other),
            },
        }
    }
}

impl PartialOrd for Gap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Gap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gap::Laps(laps) => write!(f, "{}L", laps),
            Gap::Time(time) => write!(f, "{}", time),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str_laps() {
        let gap = Gap::try_from("+1L").unwrap();
        assert_eq!(gap, Gap::Laps(1));
    }

    #[test]
    fn test_from_str_time() {
        let gap = Gap::try_from("1.234").unwrap();
        assert_eq!(gap, Gap::Time(LapTime::new(0, 1, 234)));
        let gap = Gap::try_from("+2.100").unwrap();
        assert_eq!(gap, Gap::Time(LapTime::new(0, 2, 100)));
    }
}
