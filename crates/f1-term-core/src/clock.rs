use std::{fmt::Display, time::Duration};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_remaining: Duration,
    pub updated_at: DateTime<Utc>,
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elapsed = (Utc::now() - self.updated_at)
            .to_std()
            .map_err(|_| std::fmt::Error)?;
        let remaining = self.time_remaining.saturating_sub(elapsed);

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
        };
        assert_eq!(clock.to_string(), "00:00:00");

        let clock = Clock {
            time_remaining: Duration::from_millis(1321500), // 22 minutes, 1 second (plus 500ms buffer)
            updated_at: Utc::now(),
        };
        assert_eq!(clock.to_string(), "00:22:01");

        let clock = Clock {
            time_remaining: Duration::from_millis(3725500), // 1 hour, 2 minutes, 5 seconds (plus 500ms buffer)
            updated_at: Utc::now(),
        };
        assert_eq!(clock.to_string(), "01:02:05");
    }
}
