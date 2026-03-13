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
        Ok(Clock {
            time_remaining: (Duration::from_secs(total_secs)),
        })
    }
}

pub fn convert_clock(raw: &RawExtrapolatedClock) -> Result<Clock, Box<dyn std::error::Error>> {
    Clock::try_from(raw)
}
