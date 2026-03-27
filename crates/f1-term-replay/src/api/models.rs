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
    #[serde(deserialize_with = "deserialize_valid_sessions")]
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

/// Custom deserializer to ignore instances of SessionIndex that fail to parse
/// This is done because sometimes during GP weekend some sessions are missing their Path
fn deserialize_valid_sessions<'de, D>(deserializer: D) -> Result<Vec<SessionIndex>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Vec::<serde_json::Value>::deserialize(deserializer)?;
    Ok(values
        .into_iter()
        .filter_map(|val| serde_json::from_value::<SessionIndex>(val).ok())
        .collect())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_valid_sessions_skips_missing_path() {
        let json_data = json!({
            "Key": 1,
            "Name": "Test Meeting",
            "OfficialName": "Official Test",
            "Location": "Test Track",
            "Sessions": [
                {
                    "Key": 101,
                    "Type": "Practice 1",
                    "Name": "P1",
                    "Path": "/test/path/1",
                    "StartDate": "2023-01-01T10:00:00Z",
                    "EndDate": "2023-01-01T11:00:00Z"
                },
                {
                    "Key": 102,
                    "Type": "Practice 2",
                    "Name": "P2",
                    // Intentionally missing the "Path" field
                    "StartDate": "2023-01-01T14:00:00Z",
                    "EndDate": "2023-01-01T15:00:00Z"
                },
                {
                    "Key": 103,
                    "Type": "Qualifying",
                    "Name": "Q",
                    "Path": "/test/path/3",
                    "StartDate": "2023-01-02T14:00:00Z",
                    "EndDate": "2023-01-02T15:00:00Z"
                }
            ]
        });

        let meeting: MeetingIndex =
            serde_json::from_value(json_data).expect("Failed to parse MeetingIndex");

        // The middle session (102) should be skipped because it lacks a 'Path'
        assert_eq!(
            meeting.sessions.len(),
            2,
            "Should have skipped the session without a path"
        );
        assert_eq!(meeting.sessions[0].key, 101);
        assert_eq!(meeting.sessions[0].path, "/test/path/1");
        assert_eq!(meeting.sessions[1].key, 103);
        assert_eq!(meeting.sessions[1].path, "/test/path/3");
    }
}
