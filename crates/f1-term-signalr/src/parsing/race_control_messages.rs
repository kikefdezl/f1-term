use chrono::{DateTime, NaiveDateTime, Utc};
use f1_term_core::{
    flag::{Flag, FlagColor, FlagScope},
    race_control_message::{MessageCategory, RaceControlMessage},
};
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct RaceControlMessagesPayload {
    Messages: Vec<RaceControlMessagePayload>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct RaceControlMessagePayload {
    Utc: String,
    Category: String,
    Message: String,
    Flag: Option<String>,
    Scope: Option<String>,
    Sector: Option<u8>,
}

impl TryFrom<RaceControlMessagePayload> for RaceControlMessage {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: RaceControlMessagePayload) -> Result<Self> {
        let timestamp = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::parse_from_str(&value.Utc, "%Y-%m-%dT%H:%M:%S")?,
            Utc,
        );

        let category = match value.Category.as_str() {
            "Flag" => {
                let color_str = value.Flag.ok_or("Missing 'Flag' in Flag category")?;
                let color = match color_str.as_str() {
                    "GREEN" => FlagColor::Green,
                    "YELLOW" => FlagColor::Yellow,
                    "DOUBLE YELLOW" => FlagColor::DoubleYellow,
                    "RED" => FlagColor::Red,
                    "CLEAR" => FlagColor::Clear,
                    _ => return Err(format!("Unknown flag color: {}", color_str).into()),
                };

                let scope_str = value.Scope.ok_or("Missing 'Scope' in Flag category")?;
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
            message: value.Message,
        })
    }
}

pub fn parse_race_control_messages(val: &Value) -> Result<Vec<RaceControlMessage>> {
    match val {
        Value::Object(_) => {
            let payload: RaceControlMessagesPayload = RaceControlMessagesPayload::deserialize(val)?;
            payload
                .Messages
                .into_iter()
                .map(RaceControlMessage::try_from)
                .collect()
        }
        _ => Err("RaceControlMessages value is not a JSON object".into()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use f1_term_core::flag::{FlagColor, FlagScope};
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_other_message() {
        let json_data = json!({
            "Messages": [
                {
                    "Utc": "2026-02-20T06:48:12",
                    "Category": "Other",
                    "Message": "PINK HEAD PADDING MATERIAL MUST BE USED"
                }
            ],
            "_kf": true
        });

        let parsed = parse_race_control_messages(&json_data).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            parsed[0].timestamp,
            Utc.with_ymd_and_hms(2026, 2, 20, 6, 48, 12).unwrap()
        );
        assert_eq!(parsed[0].category, MessageCategory::Other);
        assert_eq!(parsed[0].message, "PINK HEAD PADDING MATERIAL MUST BE USED");
    }

    #[test]
    fn test_parse_green_track_flag() {
        let json_data = json!({
            "Messages": [
                {
                    "Utc": "2026-02-20T07:00:00",
                    "Category": "Flag",
                    "Flag": "GREEN",
                    "Scope": "Track",
                    "Message": "GREEN LIGHT - PIT EXIT OPEN"
                }
            ],
            "_kf": true
        });

        let parsed = parse_race_control_messages(&json_data).unwrap();
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
    fn test_parse_yellow_sector_flag() {
        let json_data = json!({
            "Messages": [
                {
                    "Utc": "2026-02-20T09:11:03",
                    "Category": "Flag",
                    "Flag": "YELLOW",
                    "Scope": "Sector",
                    "Sector": 11,
                    "Message": "YELLOW IN TRACK SECTOR 11"
                }
            ],
            "_kf": true
        });

        let parsed = parse_race_control_messages(&json_data).unwrap();
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
    fn test_parse_red_flag_other_category() {
        let json_data = json!({
            "Messages": [
                {
                    "Utc": "2026-02-20T09:11:25",
                    "Category": "Other",
                    "Message": "RED FLAG - RACE SUSPENDED"
                }
            ],
            "_kf": true
        });

        let parsed = parse_race_control_messages(&json_data).unwrap();
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
    fn test_parse_double_yellow_sector_flag() {
        let json_data = json!({
            "Messages": [
                {
                    "Utc": "2026-02-20T11:01:46",
                    "Category": "Flag",
                    "Flag": "DOUBLE YELLOW",
                    "Scope": "Sector",
                    "Sector": 2,
                    "Message": "DOUBLE YELLOW IN TRACK SECTOR 2"
                }
            ],
            "_kf": true
        });

        let parsed = parse_race_control_messages(&json_data).unwrap();
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
    fn test_parse_clear_track_flag() {
        let json_data = json!({
            "Messages": [
                {
                    "Utc": "2026-02-20T09:13:18",
                    "Category": "Flag",
                    "Flag": "CLEAR",
                    "Scope": "Track",
                    "Message": "TRACK CLEAR"
                }
            ],
            "_kf": true
        });

        let parsed = parse_race_control_messages(&json_data).unwrap();
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
