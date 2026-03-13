use std::{fmt::Display, time::Duration};

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_remaining: Duration,
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut seconds = self.time_remaining.as_secs();
        let hours = seconds / 3600;
        seconds -= hours * 3600;
        let minutes = seconds / 60;
        seconds -= minutes * 60;

        write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_display() {
        let clock = Clock {
            time_remaining: Duration::from_secs(0),
        };
        assert_eq!(clock.to_string(), "00:00:00");

        let clock = Clock {
            time_remaining: Duration::from_secs(1321), // 22 minutes, 1 second
        };
        assert_eq!(clock.to_string(), "00:22:01");

        let clock = Clock {
            time_remaining: Duration::from_secs(3725), // 1 hour, 2 minutes, 5 seconds
        };
        assert_eq!(clock.to_string(), "01:02:05");
    }
}
