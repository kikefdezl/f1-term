use std::time::Duration;

use f1_term_core::clock::Clock;

use crate::parsing::extrapolated_clock::RawExtrapolatedClock;

impl TryFrom<&RawExtrapolatedClock> for Clock {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: &RawExtrapolatedClock) -> Result<Self, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = value.Remaining.split(':').collect();
        if parts.len() != 3 {
            return Err(format!("expected HH:MM:SS, got {}", value.Remaining).into());
        }

        let hours: u64 = parts[0]
            .parse()
            .map_err(|e| format!("invalid hours: {e}"))?;
        let minutes: u64 = parts[1]
            .parse()
            .map_err(|e| format!("invalid minutes: {e}"))?;
        let seconds: u64 = parts[2]
            .parse()
            .map_err(|e| format!("invalid seconds: {e}"))?;

        if minutes >= 60 || seconds >= 60 {
            return Err("minutes/seconds must be < 60".into());
        }

        let total_secs = hours * 3600 + minutes * 60 + seconds;
        let updated_at = value
            .Utc
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|e| format!("invalid date format for Utc: {e}"))?;

        Ok(Clock {
            time_remaining: Duration::from_secs(total_secs),
            updated_at,
            extrapolating: value.Extrapolating,
        })
    }
}

pub fn convert_clock(raw: &RawExtrapolatedClock) -> Result<Clock, Box<dyn std::error::Error>> {
    Clock::try_from(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike, Utc};

    #[test]
    fn test_clock_conversion() {
        let raw = RawExtrapolatedClock {
            Extrapolating: true,
            Remaining: "01:02:05".to_string(),
            Utc: "2026-03-13T03:39:11.337Z".to_string(),
        };

        let clock = Clock::try_from(&raw).unwrap();
        assert_eq!(clock.time_remaining.as_secs(), 3725);
        assert_eq!(
            clock.updated_at,
            Utc.with_ymd_and_hms(2026, 3, 13, 3, 39, 11)
                .unwrap()
                .with_nanosecond(337_000_000)
                .unwrap()
        );
    }

    #[test]
    fn test_clock_conversion_invalid_time() {
        let raw = RawExtrapolatedClock {
            Extrapolating: true,
            Remaining: "01:60:05".to_string(),
            Utc: "2026-03-13T03:39:11.337Z".to_string(),
        };
        assert!(Clock::try_from(&raw).is_err());
    }

    #[test]
    fn test_clock_conversion_invalid_date() {
        let raw = RawExtrapolatedClock {
            Extrapolating: true,
            Remaining: "01:02:05".to_string(),
            Utc: "invalid-date".to_string(),
        };
        assert!(Clock::try_from(&raw).is_err());
    }
}
