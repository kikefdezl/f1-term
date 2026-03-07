use f1_term_core::track_status::TrackStatus;

use crate::parsing::track_status::RawTrackStatus;

impl TryFrom<&RawTrackStatus> for TrackStatus {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &RawTrackStatus) -> Result<Self, Self::Error> {
        Ok(TrackStatus {
            status: value.Status.parse()?,
            message: value.Message.clone(),
        })
    }
}

pub fn convert_track_status(
    raw: &RawTrackStatus,
) -> Result<TrackStatus, Box<dyn std::error::Error>> {
    TrackStatus::try_from(raw)
}
