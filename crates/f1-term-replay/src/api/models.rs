use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IndexResponse {
    pub year: u32,
    pub meetings: Vec<MeetingIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeetingIndex {
    pub key: u32,
    pub name: String,
    pub official_name: String,
    pub location: String,
    pub sessions: Vec<SessionIndex>,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub number: i32,
    #[serde(default)]
    pub country: CountryIndex,
    #[serde(default)]
    pub circuit: CircuitIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CountryIndex {
    #[serde(default)]
    pub key: u32,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CircuitIndex {
    #[serde(default)]
    pub key: u32,
    #[serde(default)]
    pub short_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionIndex {
    pub key: u32,
    pub r#type: String,
    pub name: String,
    pub path: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub number: i32,
    #[serde(default)]
    pub gmt_offset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionRootIndex {
    pub feeds: HashMap<String, FeedIndex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FeedIndex {
    pub key_frame_path: String,
    pub stream_path: String,
}
