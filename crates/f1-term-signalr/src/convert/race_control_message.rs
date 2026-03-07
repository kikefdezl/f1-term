use chrono::{DateTime, NaiveDateTime, Utc};
use f1_term_core::{
    flag::{Flag, FlagColor, FlagScope},
    race_control_message::{MessageCategory, RaceControlMessage},
};

use crate::parsing::race_control_messages::RawRaceControlMessage;

impl TryFrom<&RawRaceControlMessage> for RaceControlMessage {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &RawRaceControlMessage) -> Result<Self, Self::Error> {
        let timestamp = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::parse_from_str(&value.Utc, "%Y-%m-%dT%H:%M:%S")?,
            Utc,
        );

        let category = match value.Category.as_str() {
            "Flag" => {
                let color_str = value
                    .Flag
                    .as_ref()
                    .ok_or("Missing 'Flag' in Flag category")?;
                let color = match color_str.as_str() {
                    "GREEN" => FlagColor::Green,
                    "YELLOW" => FlagColor::Yellow,
                    "DOUBLE YELLOW" => FlagColor::DoubleYellow,
                    "RED" => FlagColor::Red,
                    "CLEAR" => FlagColor::Clear,
                    "CHEQUERED" => FlagColor::Chequered,
                    _ => return Err(format!("Unknown flag color: {}", color_str).into()),
                };

                let scope_str = value
                    .Scope
                    .as_ref()
                    .ok_or("Missing 'Scope' in Flag category")?;
                let scope = match scope_str.as_str() {
                    "Track" => FlagScope::Track,
                    "Sector" => {
                        let sector_num = value.Sector.ok_or("Missing 'Sector' for Sector scope")?;
                        FlagScope::Sector(sector_num)
                    }
                    _ => return Err(format!("Unknown flag scope: {}", scope_str).into()),
                };

                MessageCategory::Flag(Flag { color, scope })
            }
            "SafetyCar" => MessageCategory::SafetyCar,
            "Other" => {
                // Some messages containing flag information don't come categorized as "Flag".
                // We parse them here if they contain flag specific keywords to properly colorize the UI.
                if value.Message.contains("RED FLAG") {
                    MessageCategory::Flag(Flag {
                        color: FlagColor::Red,
                        scope: FlagScope::Track,
                    })
                } else {
                    MessageCategory::Other
                }
            }
            _ => return Err(format!("Unknown Category: {}", value.Category).into()),
        };

        Ok(RaceControlMessage {
            timestamp,
            category,
            message: value.Message.clone(),
        })
    }
}

pub fn convert_race_control_messages(
    raw_messages: &[RawRaceControlMessage],
) -> Result<Vec<RaceControlMessage>, Box<dyn std::error::Error>> {
    raw_messages
        .iter()
        .map(RaceControlMessage::try_from)
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use f1_term_core::{
        flag::{Flag, FlagColor, FlagScope},
        race_control_message::MessageCategory,
    };

    use super::*;

    #[test]
    fn test_convert_other_message() {
        let raw = vec![RawRaceControlMessage {
            Utc: "2026-02-20T06:48:12".to_string(),
            Category: "Other".to_string(),
            Message: "PINK HEAD PADDING MATERIAL MUST BE USED".to_string(),
            Flag: None,
            Scope: None,
            Sector: None,
        }];

        let parsed = convert_race_control_messages(&raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0].timestamp,
            Utc.with_ymd_and_hms(2026, 2, 20, 6, 48, 12).unwrap()
        );
        assert_eq!(parsed[0].category, MessageCategory::Other);
        assert_eq!(parsed[0].message, "PINK HEAD PADDING MATERIAL MUST BE USED");
    }

    #[test]
    fn test_convert_green_track_flag() {
        let raw = vec![RawRaceControlMessage {
            Utc: "2026-02-20T07:00:00".to_string(),
            Category: "Flag".to_string(),
            Flag: Some("GREEN".to_string()),
            Scope: Some("Track".to_string()),
            Sector: None,
            Message: "GREEN LIGHT - PIT EXIT OPEN".to_string(),
        }];

        let parsed = convert_race_control_messages(&raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0].timestamp,
            Utc.with_ymd_and_hms(2026, 2, 20, 7, 0, 0).unwrap()
        );
        assert_eq!(
            parsed[0].category,
            MessageCategory::Flag(Flag {
                color: FlagColor::Green,
                scope: FlagScope::Track,
            })
        );
        assert_eq!(parsed[0].message, "GREEN LIGHT - PIT EXIT OPEN");
    }

    #[test]
    fn test_convert_yellow_sector_flag() {
        let raw = vec![RawRaceControlMessage {
            Utc: "2026-02-20T09:11:03".to_string(),
            Category: "Flag".to_string(),
            Flag: Some("YELLOW".to_string()),
            Scope: Some("Sector".to_string()),
            Sector: Some(11),
            Message: "YELLOW IN TRACK SECTOR 11".to_string(),
        }];

        let parsed = convert_race_control_messages(&raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0].timestamp,
            Utc.with_ymd_and_hms(2026, 2, 20, 9, 11, 3).unwrap()
        );
        assert_eq!(
            parsed[0].category,
            MessageCategory::Flag(Flag {
                color: FlagColor::Yellow,
                scope: FlagScope::Sector(11),
            })
        );
        assert_eq!(parsed[0].message, "YELLOW IN TRACK SECTOR 11");
    }

    #[test]
    fn test_convert_red_flag_other_category() {
        let raw = vec![RawRaceControlMessage {
            Utc: "2026-02-20T09:11:25".to_string(),
            Category: "Other".to_string(),
            Message: "RED FLAG - RACE SUSPENDED".to_string(),
            Flag: None,
            Scope: None,
            Sector: None,
        }];

        let parsed = convert_race_control_messages(&raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0].timestamp,
            Utc.with_ymd_and_hms(2026, 2, 20, 9, 11, 25).unwrap()
        );
        assert_eq!(
            parsed[0].category,
            MessageCategory::Flag(Flag {
                color: FlagColor::Red,
                scope: FlagScope::Track,
            })
        );
        assert_eq!(parsed[0].message, "RED FLAG - RACE SUSPENDED");
    }

    #[test]
    fn test_convert_double_yellow_sector_flag() {
        let raw = vec![RawRaceControlMessage {
            Utc: "2026-02-20T11:01:46".to_string(),
            Category: "Flag".to_string(),
            Flag: Some("DOUBLE YELLOW".to_string()),
            Scope: Some("Sector".to_string()),
            Sector: Some(2),
            Message: "DOUBLE YELLOW IN TRACK SECTOR 2".to_string(),
        }];

        let parsed = convert_race_control_messages(&raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0].timestamp,
            Utc.with_ymd_and_hms(2026, 2, 20, 11, 1, 46).unwrap()
        );
        assert_eq!(
            parsed[0].category,
            MessageCategory::Flag(Flag {
                color: FlagColor::DoubleYellow,
                scope: FlagScope::Sector(2),
            })
        );
        assert_eq!(parsed[0].message, "DOUBLE YELLOW IN TRACK SECTOR 2");
    }

    #[test]
    fn test_convert_clear_track_flag() {
        let raw = vec![RawRaceControlMessage {
            Utc: "2026-02-20T09:13:18".to_string(),
            Category: "Flag".to_string(),
            Flag: Some("CLEAR".to_string()),
            Scope: Some("Track".to_string()),
            Sector: None,
            Message: "TRACK CLEAR".to_string(),
        }];

        let parsed = convert_race_control_messages(&raw).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0].timestamp,
            Utc.with_ymd_and_hms(2026, 2, 20, 9, 13, 18).unwrap()
        );
        assert_eq!(
            parsed[0].category,
            MessageCategory::Flag(Flag {
                color: FlagColor::Clear,
                scope: FlagScope::Track,
            })
        );
        assert_eq!(parsed[0].message, "TRACK CLEAR");
    }
}
