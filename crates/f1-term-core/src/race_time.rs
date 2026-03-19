use std::fmt::Display;

#[derive(Debug, Default, Clone)]
pub struct RaceTime {
    pub minutes: u32,
    pub seconds: u32,
    pub millis: u32,
}

impl Display for RaceTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:01}:{:02}.{:03}",
            self.minutes, self.seconds, self.millis
        )
    }
}

impl RaceTime {
    fn total_millis(&self) -> u64 {
        self.minutes as u64 * 60 * 1000 + self.seconds as u64 * 1000 + self.millis as u64
    }
}

/// Initializes the RaceTime from a &str of shape 1:23.345 or 23.345
impl TryFrom<&str> for RaceTime {
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // If there's a colon, split it. Otherwise, assume 0 minutes.
        let (minutes_str, rest) = s.split_once(':').unwrap_or(("0", s));
        let (seconds_str, millis_str) =
            rest.split_once('.').ok_or("Missing dot separator ('.')")?;

        Ok(RaceTime {
            minutes: minutes_str.parse()?,
            seconds: seconds_str.parse()?,
            millis: millis_str.parse()?,
        })
    }
}

impl PartialEq for RaceTime {
    fn eq(&self, other: &Self) -> bool {
        self.total_millis() == other.total_millis()
    }
}

impl Eq for RaceTime {}

impl PartialOrd for RaceTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RaceTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_millis().cmp(&other.total_millis())
    }
}
