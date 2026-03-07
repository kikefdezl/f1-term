use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawSessionInfo {
    pub Meeting: RawMeeting,
    pub SessionStatus: String,
    pub ArchiveStatus: RawArchiveStatus,
    pub Key: u32,
    pub Type: String,
    pub Number: Option<String>,
    pub Name: String,
    pub StartDate: String,
    pub EndDate: String,
    pub GmtOffset: String,
    pub Path: String,
    pub _kf: bool,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawMeeting {
    pub Key: u32,
    pub Name: String,
    pub OfficialName: String,
    pub Location: String,
    pub Number: u32,
    pub Country: RawCountry,
    pub Circuit: RawCircuit,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawArchiveStatus {
    pub Status: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawCountry {
    pub Key: u32,
    pub Code: String,
    pub Name: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawCircuit {
    pub Key: u32,
    pub ShortName: String,
}

pub fn parse_raw_session_info(info_val: &Value) -> Result<RawSessionInfo> {
    match info_val {
        Value::Object(_) => match RawSessionInfo::deserialize(info_val) {
            Ok(sip) => Ok(sip),
            Err(e) => Err(format!("Failed to parse RawSessionInfo: {}", e).into()),
        },
        _ => Err("SessionInfo value is not a JSON object".into()),
    }
}
