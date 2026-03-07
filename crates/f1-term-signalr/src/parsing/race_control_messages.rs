use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawRaceControlMessages {
    pub Messages: Vec<RawRaceControlMessage>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawRaceControlMessage {
    pub Utc: String,
    pub Category: String,
    pub Message: String,
    pub Flag: Option<String>,
    pub Scope: Option<String>,
    pub Sector: Option<u8>,
}

pub fn parse_raw_race_control_messages(val: &Value) -> Result<RawRaceControlMessages> {
    match val {
        Value::Object(_) => {
            let payload: RawRaceControlMessages = RawRaceControlMessages::deserialize(val)?;
            Ok(payload)
        }
        _ => Err("RaceControlMessages value is not a JSON object".into()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use f1_term_core::{
        flag::{Flag, FlagColor, FlagScope},
        race_control_message::MessageCategory,
    };
    use serde_json::json;

    use super::*;
    use crate::convert::race_control_message::convert_race_control_messages;

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

        let raw = parse_raw_race_control_messages(&json_data).unwrap();
        let parsed = convert_race_control_messages(&raw.Messages).unwrap();
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

        let raw = parse_raw_race_control_messages(&json_data).unwrap();
        let parsed = convert_race_control_messages(&raw.Messages).unwrap();
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

        let raw = parse_raw_race_control_messages(&json_data).unwrap();
        let parsed = convert_race_control_messages(&raw.Messages).unwrap();
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

        let raw = parse_raw_race_control_messages(&json_data).unwrap();
        let parsed = convert_race_control_messages(&raw.Messages).unwrap();
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

        let raw = parse_raw_race_control_messages(&json_data).unwrap();
        let parsed = convert_race_control_messages(&raw.Messages).unwrap();
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

        let raw = parse_raw_race_control_messages(&json_data).unwrap();
        let parsed = convert_race_control_messages(&raw.Messages).unwrap();
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
