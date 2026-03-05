use f1_term_core::track_status::TrackStatus;
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case)]
struct TrackStatusPayload {
    Status: String,
    Message: String,
}

impl TryFrom<TrackStatusPayload> for TrackStatus {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: TrackStatusPayload) -> Result<Self> {
        Ok(TrackStatus {
            status: value.Status.parse()?,
            message: value.Message.clone(),
        })
    }
}

pub fn parse_track_status(val: &Value) -> Result<TrackStatus> {
    let payload: TrackStatusPayload = TrackStatusPayload::deserialize(val)?;
    TrackStatus::try_from(payload)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_track_status_valid() {
        let json_val = json!({
            "Status": "1",
            "Message": "AllClear"
        });

        let result = parse_track_status(&json_val);
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.status, 1);
        assert_eq!(status.message, "AllClear");
    }

    #[test]
    fn test_parse_track_status_invalid_status_type() {
        let json_val = json!({
            "Status": "not_a_number",
            "Message": "Clear"
        });

        let result = parse_track_status(&json_val);
        assert!(result.is_err());
    }
}
