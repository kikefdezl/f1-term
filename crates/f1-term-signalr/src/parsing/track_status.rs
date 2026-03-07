use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
pub struct RawTrackStatus {
    pub Status: String,
    pub Message: String,
}

pub fn parse_raw_track_status(val: &Value) -> Result<RawTrackStatus> {
    let payload = RawTrackStatus::deserialize(val)?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::convert::track_status::convert_track_status;

    #[test]
    fn test_parse_track_status_valid() {
        let json_val = json!({
            "Status": "1",
            "Message": "AllClear"
        });

        let raw = parse_raw_track_status(&json_val).unwrap();
        let status = convert_track_status(&raw).unwrap();
        assert_eq!(status.status, 1);
        assert_eq!(status.message, "AllClear");
    }

    #[test]
    fn test_parse_track_status_invalid_status_type() {
        let json_val = json!({
            "Status": "not_a_number",
            "Message": "Clear"
        });

        let raw = parse_raw_track_status(&json_val).unwrap();
        let result = convert_track_status(&raw);
        assert!(result.is_err());
    }
}
