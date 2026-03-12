use f1_term_core::circuit::{Circuit, CircuitKey};

use crate::parsing::session_info::RawSessionInfo;

impl From<&RawSessionInfo> for Circuit {
    fn from(value: &RawSessionInfo) -> Self {
        use chrono::Datelike;
        let year = chrono::DateTime::parse_from_rfc3339(&value.StartDate)
            .map(|dt| dt.year() as u32)
            .unwrap_or_else(|_| chrono::Utc::now().year() as u32);

        Circuit {
            key: CircuitKey(value.Meeting.Circuit.Key),
            short_name: value.Meeting.Circuit.ShortName.clone(),
            layout: None,
            year,
        }
    }
}

pub fn convert_circuit(raw_info: &RawSessionInfo) -> Circuit {
    Circuit::from(raw_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::session_info::{RawArchiveStatus, RawCircuit, RawCountry, RawMeeting};

    #[test]
    fn test_convert_circuit() {
        let raw_info = RawSessionInfo {
            Meeting: RawMeeting {
                Key: 1,
                Name: "Meeting".into(),
                OfficialName: "Official".into(),
                Location: "Location".into(),
                Number: 1,
                Country: RawCountry {
                    Key: 1,
                    Code: "C".into(),
                    Name: "Country".into(),
                },
                Circuit: RawCircuit {
                    Key: 42,
                    ShortName: "Silverstone".into(),
                },
            },
            SessionStatus: "Active".into(),
            ArchiveStatus: RawArchiveStatus {
                Status: "Archived".into(),
            },
            Key: 1,
            Type: "Race".into(),
            Number: None,
            Name: "Race".into(),
            StartDate: "2024-07-07T14:00:00Z".into(),
            EndDate: "2024-07-07T16:00:00Z".into(),
            GmtOffset: "+01:00".into(),
            Path: "path".into(),
            _kf: true,
        };

        let circuit = convert_circuit(&raw_info);
        assert_eq!(circuit.key.0, 42);
        assert_eq!(circuit.short_name, "Silverstone");
        assert_eq!(circuit.year, 2024);
        assert!(circuit.layout.is_none());
    }
}
