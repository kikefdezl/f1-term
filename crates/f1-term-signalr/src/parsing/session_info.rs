use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawSessionInfo {
    pub Meeting: RawMeeting,
    pub SessionStatus: String,
    pub ArchiveStatus: RawArchiveStatus,
    pub Key: u32,
    pub Type: String,
    pub Number: Option<u8>,
    pub Name: String,
    pub StartDate: String,
    pub EndDate: String,
    pub GmtOffset: String,
    pub Path: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawMeeting {
    pub Key: u32,
    pub Name: String,
    pub OfficialName: String,
    pub Location: String,
    pub Number: u32,
    pub Country: RawCountry,
    pub Circuit: RawCircuit,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawArchiveStatus {
    pub Status: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawCountry {
    pub Key: u32,
    pub Code: String,
    pub Name: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawCircuit {
    pub Key: u32,
    pub ShortName: String,
}

pub fn parse_raw_session_info(info_val: &Value) -> Result<RawSessionInfo> {
    match info_val {
        Value::Object(_) => match RawSessionInfo::deserialize(info_val) {
            Ok(sip) => Ok(sip),
            Err(e) => Err(format!("Failed to parse RawSessionInfo: {}", e).into()),
        },
        _ => Err("SessionInfo value is not a JSON object".into()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_raw_session_info() {
        let json_data = json!({
            "Meeting": {
                "Key": 1234,
                "Name": "Bahrain Grand Prix",
                "OfficialName": "FORMULA 1 GULF AIR BAHRAIN GRAND PRIX 2024",
                "Location": "Sakhir",
                "Number": 1,
                "Country": {
                    "Key": 36,
                    "Code": "BRN",
                    "Name": "Bahrain"
                },
                "Circuit": {
                    "Key": 3,
                    "ShortName": "Bahrain International Circuit"
                }
            },
            "SessionStatus": "Active",
            "ArchiveStatus": {
                "Status": "Generating"
            },
            "Key": 9123,
            "Type": "Practice",
            "Number": 1,
            "Name": "Practice 1",
            "StartDate": "2024-03-02T15:00:00Z",
            "EndDate": "2024-03-02T17:00:00Z",
            "GmtOffset": "03:00:00",
            "Path": "2024/2024-03-02_Race",
            "_kf": true
        });

        let result = parse_raw_session_info(&json_data).expect("Failed to parse raw session info");

        assert_eq!(result.Key, 9123);
        assert_eq!(result.Type, "Practice");
        assert_eq!(result.Number, Some(1));
        assert_eq!(result.Name, "Practice 1");
        assert_eq!(result.StartDate, "2024-03-02T15:00:00Z");
        assert_eq!(result.EndDate, "2024-03-02T17:00:00Z");
        assert_eq!(result.GmtOffset, "03:00:00");
        assert_eq!(result.Path, "2024/2024-03-02_Race");
        assert_eq!(result.SessionStatus, "Active");
        assert_eq!(result.ArchiveStatus.Status, "Generating");
        assert_eq!(result.Meeting.Key, 1234);
        assert_eq!(result.Meeting.Name, "Bahrain Grand Prix");
        assert_eq!(
            result.Meeting.OfficialName,
            "FORMULA 1 GULF AIR BAHRAIN GRAND PRIX 2024"
        );
        assert_eq!(result.Meeting.Location, "Sakhir");
        assert_eq!(result.Meeting.Number, 1);
        assert_eq!(result.Meeting.Country.Key, 36);
        assert_eq!(result.Meeting.Country.Code, "BRN");
        assert_eq!(result.Meeting.Country.Name, "Bahrain");
        assert_eq!(result.Meeting.Circuit.Key, 3);
        assert_eq!(
            result.Meeting.Circuit.ShortName,
            "Bahrain International Circuit"
        );
    }
}
