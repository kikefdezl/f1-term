use chrono::{DateTime, Utc};

use super::flag::Flag;

#[derive(Debug, Clone, PartialEq)]
pub struct RaceControlMessage {
    pub timestamp: DateTime<Utc>,
    pub category: MessageCategory,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageCategory {
    Flag(Flag),
    Other,
}
