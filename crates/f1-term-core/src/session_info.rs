#[derive(Debug)]
pub struct SessionInfo {
    pub meeting: Meeting,
    pub session_status: SessionStatus,
    pub archive_status: ArchiveStatus,
    pub key: u32,
    pub type_: SessionType,
    pub number: u8,
    pub name: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub utc_offset: chrono::FixedOffset,
    pub path: String,
}

#[derive(Debug, Default)]
pub struct Meeting {
    pub key: u32,
    pub name: String,
    pub official_name: String,
    pub location: String,
    pub number: u8,
    pub country: Country,
    pub circuit: Circuit,
}

#[derive(Debug, Default)]
pub struct Country {
    pub key: u32,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct Circuit {
    pub key: u32,
    pub short_name: String,
}

#[derive(Debug)]
pub enum SessionStatus {
    Finalised,
}

#[derive(Debug)]
pub enum ArchiveStatus {
    Complete,
}

#[derive(Debug)]
pub enum SessionType {
    Practice,
}
