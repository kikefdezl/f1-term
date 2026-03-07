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

        let raw = parse_raw_race_control_messages(&json_data).unwrap();
        assert_eq!(raw.Messages.len(), 1);
        assert_eq!(raw.Messages[0].Utc, "2026-02-20T06:48:12");
        assert_eq!(raw.Messages[0].Category, "Other");
        assert_eq!(
            raw.Messages[0].Message,
            "PINK HEAD PADDING MATERIAL MUST BE USED"
        );
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
        assert_eq!(raw.Messages.len(), 1);
        assert_eq!(raw.Messages[0].Utc, "2026-02-20T07:00:00");
        assert_eq!(raw.Messages[0].Category, "Flag");
        assert_eq!(raw.Messages[0].Flag.as_deref(), Some("GREEN"));
        assert_eq!(raw.Messages[0].Scope.as_deref(), Some("Track"));
        assert_eq!(raw.Messages[0].Message, "GREEN LIGHT - PIT EXIT OPEN");
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
        assert_eq!(raw.Messages.len(), 1);
        assert_eq!(raw.Messages[0].Utc, "2026-02-20T09:11:03");
        assert_eq!(raw.Messages[0].Category, "Flag");
        assert_eq!(raw.Messages[0].Flag.as_deref(), Some("YELLOW"));
        assert_eq!(raw.Messages[0].Scope.as_deref(), Some("Sector"));
        assert_eq!(raw.Messages[0].Sector, Some(11));
        assert_eq!(raw.Messages[0].Message, "YELLOW IN TRACK SECTOR 11");
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
        assert_eq!(raw.Messages.len(), 1);
        assert_eq!(raw.Messages[0].Utc, "2026-02-20T09:11:25");
        assert_eq!(raw.Messages[0].Category, "Other");
        assert_eq!(raw.Messages[0].Message, "RED FLAG - RACE SUSPENDED");
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
        assert_eq!(raw.Messages.len(), 1);
        assert_eq!(raw.Messages[0].Utc, "2026-02-20T11:01:46");
        assert_eq!(raw.Messages[0].Category, "Flag");
        assert_eq!(raw.Messages[0].Flag.as_deref(), Some("DOUBLE YELLOW"));
        assert_eq!(raw.Messages[0].Scope.as_deref(), Some("Sector"));
        assert_eq!(raw.Messages[0].Sector, Some(2));
        assert_eq!(raw.Messages[0].Message, "DOUBLE YELLOW IN TRACK SECTOR 2");
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
        assert_eq!(raw.Messages.len(), 1);
        assert_eq!(raw.Messages[0].Utc, "2026-02-20T09:13:18");
        assert_eq!(raw.Messages[0].Category, "Flag");
        assert_eq!(raw.Messages[0].Flag.as_deref(), Some("CLEAR"));
        assert_eq!(raw.Messages[0].Scope.as_deref(), Some("Track"));
        assert_eq!(raw.Messages[0].Message, "TRACK CLEAR");
    }
}
