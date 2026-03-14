use std::{fmt::Display, time::Duration};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_remaining: Duration,
    pub updated_at: DateTime<Utc>,
    pub extrapolating: bool,
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let remaining = if self.extrapolating {
            let elapsed_since = (Utc::now() - self.updated_at)
                .to_std()
                .map_err(|_| std::fmt::Error)?;
            self.time_remaining.saturating_sub(elapsed_since)
        } else {
            self.time_remaining
        };

        let mut seconds = remaining.as_secs();
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
            updated_at: Utc::now(),
            extrapolating: false,
        };
        assert_eq!(clock.to_string(), "00:00:00");

        let clock = Clock {
            time_remaining: Duration::from_secs(3725), // 1 hour, 2 minutes, 5 seconds
            updated_at: Utc::now(),
            extrapolating: false,
        };
        assert_eq!(clock.to_string(), "01:02:05");

        let clock = Clock {
            time_remaining: Duration::from_millis(1321500), // 22 minutes, 1 second (plus 500ms buffer)
            updated_at: Utc::now(),
            extrapolating: true,
        };
        assert_eq!(clock.to_string(), "00:22:01");
    }
}
