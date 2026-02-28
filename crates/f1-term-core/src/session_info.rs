use std::fmt::Display;

use chrono::{DateTime, FixedOffset, Utc};

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub meeting: Meeting,
    pub session_status: SessionStatus,
    pub archive_status: ArchiveStatus,
    pub key: u32,
    pub type_: SessionType,
    pub number: u8,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub utc_offset: FixedOffset,
    pub path: String,
}

#[derive(Debug, Default, Clone)]
pub struct Meeting {
    pub key: u32,
    pub name: String,
    pub official_name: String,
    pub location: String,
    pub number: u8,
    pub country: Country,
    pub circuit: Circuit,
}

#[derive(Debug, Default, Clone)]
pub struct Country {
    pub key: u32,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub struct Circuit {
    pub key: u32,
    pub short_name: String,
}

#[derive(Debug, Clone)]
pub enum SessionStatus {
    Finalised,
}

#[derive(Debug, Clone)]
pub enum ArchiveStatus {
    Complete,
}

#[derive(Debug, Clone)]
pub enum SessionType {
    Practice,
}

impl Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Practice => write!(f, "Practice"),
        }
    }
}
