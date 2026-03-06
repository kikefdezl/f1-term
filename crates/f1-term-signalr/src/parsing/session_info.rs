use std::convert::TryFrom;

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use f1_term_core::{
    circuit::Circuit,
    session_info::{ArchiveStatus, Country, Meeting, SessionInfo, SessionStatus, SessionType},
};
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct SessionInfoPayload {
    Meeting: MeetingPayload,
    SessionStatus: String,
    ArchiveStatus: ArchiveStatusPayload,
    Key: u32,
    Type: String,
    Number: u32,
    Name: String,
    StartDate: String,
    EndDate: String,
    GmtOffset: String,
    Path: String,
    _kf: bool,
}

impl TryFrom<SessionInfoPayload> for SessionInfo {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: SessionInfoPayload) -> Result<Self> {
        let session_status = match value.SessionStatus.as_str() {
            "Started" => SessionStatus::Started,
            "Finalised" => SessionStatus::Finalised,
            _ => return Err(format!("Unknown SessionStatus: {}", value.SessionStatus).into()),
        };

        let archive_status = match value.ArchiveStatus.Status.as_str() {
            "Generating" => ArchiveStatus::Generating,
            "Complete" => ArchiveStatus::Complete,
            _ => {
                return Err(
                    format!("Unknown ArchiveStatus: {}", value.ArchiveStatus.Status).into(),
                );
            }
        };

        let session_type = match value.Type.as_str() {
            "Practice" => SessionType::Practice,
            _ => return Err(format!("Unknown SessionType: {}", value.Type).into()),
        };

        let offset_parts: Vec<i32> = value
            .GmtOffset
            .split(':')
            .map(|p| {
                p.parse::<i32>()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            })
            .collect::<Result<_>>()?;

        let utc_offset =
            FixedOffset::east_opt(offset_parts[0] * 3600 + offset_parts[1] * 60 + offset_parts[2])
                .ok_or("Invalid UTC offset")?;

        Ok(SessionInfo {
            meeting: Meeting::try_from(value.Meeting)?,
            session_status,
            archive_status,
            key: value.Key,
            type_: session_type,
            number: value.Number.try_into()?,
            name: value.Name,
            start_date: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(&value.StartDate, "%Y-%m-%dT%H:%M:%S")?,
                Utc,
            ),
            end_date: DateTime::from_naive_utc_and_offset(
                NaiveDateTime::parse_from_str(&value.EndDate, "%Y-%m-%dT%H:%M:%S")?,
                Utc,
            ),
            utc_offset,
            path: value.Path,
        })
    }
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct MeetingPayload {
    Key: u32,
    Name: String,
    OfficialName: String,
    Location: String,
    Number: u32,
    Country: CountryPayload,
    Circuit: CircuitPayload,
}

impl TryFrom<MeetingPayload> for Meeting {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: MeetingPayload) -> Result<Self> {
        Ok(Meeting {
            key: value.Key,
            name: value.Name,
            official_name: value.OfficialName,
            location: value.Location,
            number: value.Number.try_into()?,
            country: Country::try_from(value.Country)?,
            circuit: Circuit::try_from(value.Circuit)?,
        })
    }
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct ArchiveStatusPayload {
    Status: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct CountryPayload {
    Key: u32,
    Code: String,
    Name: String,
}

impl TryFrom<CountryPayload> for Country {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: CountryPayload) -> Result<Self> {
        Ok(Country {
            key: value.Key,
            code: value.Code,
            name: value.Name,
        })
    }
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct CircuitPayload {
    Key: u32,
    ShortName: String,
}

impl TryFrom<CircuitPayload> for Circuit {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: CircuitPayload) -> Result<Self> {
        Ok(Circuit {
            key: value.Key,
            short_name: value.ShortName,
            layout: None,
        })
    }
}

pub fn parse_session_info(val: &Value) -> Result<SessionInfo> {
    match val {
        Value::Object(_) => match SessionInfoPayload::deserialize(val) {
            Ok(sip) => SessionInfo::try_from(sip),
            Err(_) => Err("Failed to parse SessionInfoPayload".into()),
        },
        _ => Err("SessionInfo value is not a JSON object".into()),
    }
}
