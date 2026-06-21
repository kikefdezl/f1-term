use std::time::Duration;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_remaining: Duration,
    pub updated_at: DateTime<Utc>,
    pub extrapolating: bool,
}
