use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use f1_term_core::{
    circuit::CircuitKey,
    session_info::{
        ArchiveStatus, Country, Meeting, QualiPhase, SessionInfo, SessionStatus, SessionType,
    },
};

use crate::parsing::{
    session_data::RawSessionData,
    session_info::{RawCountry, RawMeeting, RawSessionInfo},
};

impl TryFrom<&RawCountry> for Country {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: &RawCountry) -> Result<Self, Self::Error> {
        Ok(Country {
            key: value.Key,
            code: value.Code.clone(),
            name: value.Name.clone(),
        })
    }
}

impl TryFrom<&RawMeeting> for Meeting {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &RawMeeting) -> Result<Self, Self::Error> {
        Ok(Meeting {
            key: value.Key,
            name: value.Name.clone(),
            official_name: value.OfficialName.clone(),
            location: value.Location.clone(),
            number: value.Number.try_into()?,
            country: Country::try_from(&value.Country)?,
            circuit_key: CircuitKey(value.Circuit.Key),
        })
    }
}

pub fn convert_session_info(
    raw_info: &RawSessionInfo,
    raw_data: Option<&RawSessionData>,
) -> Result<SessionInfo, Box<dyn std::error::Error>> {
    let quali_part = raw_data.and_then(|data| {
        data.Series
            .as_ref()
            .and_then(|series| series.iter().rev().find_map(|s| s.QualifyingPart))
    });
    let session_status = match raw_info.SessionStatus.as_str() {
        "Inactive" => SessionStatus::Inactive,
        "Started" => SessionStatus::Started,
        "Finished" => SessionStatus::Started,
        "Finalised" => SessionStatus::Finalised,
        _ => return Err(format!("Unknown SessionStatus: {}", raw_info.SessionStatus).into()),
    };

    let archive_status = match raw_info.ArchiveStatus.Status.as_str() {
        "Generating" => ArchiveStatus::Generating,
        "Complete" => ArchiveStatus::Complete,
        _ => {
            return Err(format!("Unknown ArchiveStatus: {}", raw_info.ArchiveStatus.Status).into());
        }
    };

    let session_type = match raw_info.Type.as_str() {
        "Practice" => SessionType::Practice,
        "Qualifying" => {
            let phase = quali_part.and_then(|p| match p {
                1 => Some(QualiPhase::Q1),
                2 => Some(QualiPhase::Q2),
                3 => Some(QualiPhase::Q3),
                _ => None,
            });
            SessionType::Qualifying(phase)
        }
        "Race" => SessionType::Race,
        _ => return Err(format!("Unknown SessionType: {}", raw_info.Type).into()),
    };

    let offset_parts: Vec<i32> = raw_info
        .GmtOffset
        .split(':')
        .map(|p: &str| {
            p.parse::<i32>()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
        .collect::<Result<_, _>>()?;

    let utc_offset =
        FixedOffset::east_opt(offset_parts[0] * 3600 + offset_parts[1] * 60 + offset_parts[2])
            .ok_or("Invalid UTC offset")?;

    Ok(SessionInfo {
        meeting: Meeting::try_from(&raw_info.Meeting)?,
        session_status,
        archive_status,
        key: raw_info.Key,
        type_: session_type,
        number: raw_info.Number,
        name: raw_info.Name.clone(),
        start_date: DateTime::from_naive_utc_and_offset(
            NaiveDateTime::parse_from_str(&raw_info.StartDate, "%Y-%m-%dT%H:%M:%S")?,
            Utc,
        ),
        end_date: DateTime::from_naive_utc_and_offset(
            NaiveDateTime::parse_from_str(&raw_info.EndDate, "%Y-%m-%dT%H:%M:%S")?,
            Utc,
        ),
        utc_offset,
        path: raw_info.Path.clone(),
    })
}
