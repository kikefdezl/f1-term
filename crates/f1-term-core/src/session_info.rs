use std::fmt::Display;

use chrono::{DateTime, FixedOffset, Utc};

use super::circuit::Circuit;

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub meeting: Meeting,
    pub session_status: SessionStatus,
    pub archive_status: ArchiveStatus,
    pub key: u32,
    pub type_: SessionType,
    pub number: Option<u8>,
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

#[derive(Debug, Clone)]
pub enum SessionStatus {
    Started,
    Finished,
    Finalised,
}

#[derive(Debug, Clone)]
pub enum ArchiveStatus {
    Generating,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualiPhase {
    Q1,
    Q2,
    Q3,
}

impl QualiPhase {
    pub fn index(&self) -> usize {
        match self {
            QualiPhase::Q1 => 0,
            QualiPhase::Q2 => 1,
            QualiPhase::Q3 => 2,
        }
    }
}

impl Display for QualiPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q1 => write!(f, "Q1"),
            Self::Q2 => write!(f, "Q2"),
            Self::Q3 => write!(f, "Q3"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionType {
    Practice,
    Qualifying(Option<QualiPhase>),
    Race,
}

impl Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Practice => write!(f, "Practice"),
            Self::Qualifying(Some(phase)) => write!(f, "Qualifying - {}", phase),
            Self::Qualifying(None) => write!(f, "Qualifying"),
            Self::Race => write!(f, "Race"),
        }
    }
}
