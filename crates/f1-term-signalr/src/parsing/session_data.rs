use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct SessionDataPayload {
    #[serde(default)]
    Series: Option<Vec<SeriesPayload>>,
    #[serde(default)]
    StatusSeries: Option<Vec<StatusSeriesPayload>>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct SeriesPayload {
    QualifyingPart: Option<u8>,
    Utc: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct StatusSeriesPayload {
    TrackStatus: Option<String>,
    SessionStatus: Option<String>,
    Utc: Option<String>,
}

pub fn qualifying_part(data: &serde_json::Value) -> Option<u8> {
    if let Ok(payload) = serde_json::from_value::<SessionDataPayload>(data.clone()) {
        payload
            .Series
            .and_then(|series| series.into_iter().rev().find_map(|s| s.QualifyingPart))
    } else {
        None
    }
}
