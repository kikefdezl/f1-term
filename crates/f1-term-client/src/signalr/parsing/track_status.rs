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
    let payload: TrackStatusPayload = serde_json::from_value(val.clone())?;
    TrackStatus::try_from(payload)
}
