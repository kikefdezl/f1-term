use std::fmt::Display;

#[derive(Debug, Default, Clone, Eq, PartialEq, Copy)]
pub struct LapTime {
    pub minutes: u32,
    pub seconds: u32,
    pub millis: u32,
}

impl LapTime {
    pub fn new(minutes: u32, seconds: u32, millis: u32) -> Self {
        LapTime {
            minutes,
            seconds,
            millis,
        }
    }

    pub fn from_seconds(seconds: u32) -> Self {
        LapTime {
            minutes: 0,
            seconds,
            millis: 0,
        }
    }

    pub fn from_millis(millis: u32) -> Self {
        LapTime {
            minutes: 0,
            seconds: 0,
            millis,
        }
    }

    pub fn millis(&self) -> u64 {
        self.minutes as u64 * 60 * 1000 + self.seconds as u64 * 1000 + self.millis as u64
    }
}

impl Display for LapTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.minutes == 0 {
            write!(f, "{}.{:03}", self.seconds, self.millis)
        } else {
            write!(f, "{}:{:02}.{:03}", self.minutes, self.seconds, self.millis)
        }
    }
}

/// Initializes the RaceTime from a &str of shape 1:23.345 or 23.345
impl TryFrom<&str> for LapTime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // If there's a colon, split it. Otherwise, assume 0 minutes.
        let (minutes_str, rest) = s.split_once(':').unwrap_or(("0", s));
        let (seconds_str, millis_str) =
            rest.split_once('.').ok_or("Missing dot separator ('.')")?;

        Ok(LapTime {
            minutes: minutes_str.parse()?,
            seconds: seconds_str.parse()?,
            millis: millis_str.parse()?,
        })
    }
}

impl PartialOrd for LapTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LapTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.millis().cmp(&other.millis())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disp() {
        let time = LapTime::new(1, 23, 800);
        assert_eq!(time.to_string(), "1:23.800".to_string());
        let time = LapTime::new(1, 3, 8);
        assert_eq!(time.to_string(), "1:03.008".to_string());
        let time = LapTime::new(0, 32, 845);
        assert_eq!(time.to_string(), "32.845".to_string());
    }
}
