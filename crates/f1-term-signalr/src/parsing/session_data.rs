use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct RawSessionData {
    #[serde(default)]
    pub Series: Option<Vec<RawSeries>>,
    #[serde(default)]
    pub StatusSeries: Option<Vec<RawStatusSeries>>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct RawSeries {
    pub QualifyingPart: Option<u8>,
    pub Utc: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct RawStatusSeries {
    pub TrackStatus: Option<String>,
    pub SessionStatus: Option<String>,
    pub Utc: Option<String>,
}

pub fn parse_raw_session_data(data: &serde_json::Value) -> Option<RawSessionData> {
    serde_json::from_value::<RawSessionData>(data.clone()).ok()
}
