use std::collections::HashSet;

use chrono::Datelike;
use f1_term_core::circuit::{Circuit, CircuitKey, CircuitScope, CircuitStatus};

use crate::parsing::{race_control_messages::RawRaceControlMessages, session_info::RawSessionInfo};

pub fn convert_circuit(raw_info: &RawSessionInfo, rcm: Option<&RawRaceControlMessages>) -> Circuit {
    let year = chrono::DateTime::parse_from_rfc3339(&raw_info.StartDate)
        .map(|dt| dt.year() as u32)
        .unwrap_or_else(|_| chrono::Utc::now().year() as u32);

    Circuit {
        key: CircuitKey(raw_info.Meeting.Circuit.Key),
        short_name: raw_info.Meeting.Circuit.ShortName.clone(),
        layout: None,
        year,
        status: compute_circuit_status(rcm),
    }
}

pub fn compute_circuit_status(rcm: Option<&RawRaceControlMessages>) -> CircuitStatus {
    let mut status = CircuitStatus::Clear;
    let mut yellow_sectors: HashSet<u8> = HashSet::new();

    // This algorithm goes through all the messages to compute the current status of
    // the track.
    // It goes from old -> new because you might have sequential yellow flags that
    // have to be aggregated (e.g. first a yellow flag on sector 1 and then one on
    // sector 2 & 3)
    if let Some(messages) = rcm {
        for msg in &messages.Messages {
            if msg.Category == "Flag" {
                match msg.Flag.as_deref() {
                    Some("GREEN") | Some("CLEAR") => match msg.Scope.as_deref() {
                        Some("Sector") => {
                            if let Some(sector) = msg.Sector {
                                yellow_sectors.remove(&sector);
                                if yellow_sectors.is_empty() {
                                    status = CircuitStatus::Clear;
                                } else {
                                    let mut sectors: Vec<u8> =
                                        yellow_sectors.iter().copied().collect();
                                    sectors.sort_unstable();
                                    status = CircuitStatus::Yellow(CircuitScope::Sectors(sectors));
                                }
                            }
                        }
                        _ => {
                            status = CircuitStatus::Clear;
                            yellow_sectors.clear();
                        }
                    },
                    Some("YELLOW") | Some("DOUBLE YELLOW") => match msg.Scope.as_deref() {
                        Some("Sector") => {
                            if let Some(sector) = msg.Sector {
                                yellow_sectors.insert(sector);
                                let mut sectors: Vec<u8> = yellow_sectors.iter().copied().collect();
                                sectors.sort_unstable();
                                status = CircuitStatus::Yellow(CircuitScope::Sectors(sectors));
                            }
                        }
                        Some("Track") => {
                            status = CircuitStatus::Yellow(CircuitScope::Full);
                            yellow_sectors.clear();
                        }
                        _ => {}
                    },
                    Some("RED") => {
                        status = CircuitStatus::Red;
                        yellow_sectors.clear();
                    }
                    _ => {}
                }
            } else if msg.Message.contains("RED FLAG") {
                status = CircuitStatus::Red;
                yellow_sectors.clear();
            }
        }
    }

    status
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

        let circuit = convert_circuit(&raw_info, None);
        assert_eq!(circuit.key.0, 42);
        assert_eq!(circuit.short_name, "Silverstone");
        assert_eq!(circuit.year, 2024);
        assert!(circuit.layout.is_none());
        assert!(matches!(circuit.status, CircuitStatus::Clear));
    }
}

#[cfg(test)]
mod rcm_tests {
    use super::*;
    use crate::parsing::race_control_messages::RawRaceControlMessage;

    #[test]
    fn test_track_status_green() {
        let rcm = RawRaceControlMessages {
            Messages: vec![RawRaceControlMessage {
                Utc: "time".into(),
                Category: "Flag".into(),
                Message: "GREEN LIGHT".into(),
                Flag: Some("GREEN".into()),
                Scope: Some("Track".into()),
                Sector: None,
            }],
        };
        let status = compute_circuit_status(Some(&rcm));
        assert!(matches!(status, CircuitStatus::Clear));
    }

    #[test]
    fn test_track_status_red() {
        let rcm = RawRaceControlMessages {
            Messages: vec![RawRaceControlMessage {
                Utc: "time".into(),
                Category: "Flag".into(),
                Message: "RED FLAG".into(),
                Flag: Some("RED".into()),
                Scope: Some("Track".into()),
                Sector: None,
            }],
        };
        let status = compute_circuit_status(Some(&rcm));
        assert!(matches!(status, CircuitStatus::Red));
    }

    #[test]
    fn test_track_status_sector_yellows() {
        let rcm = RawRaceControlMessages {
            Messages: vec![
                RawRaceControlMessage {
                    Utc: "time".into(),
                    Category: "Flag".into(),
                    Message: "YELLOW IN TRACK SECTOR 11".into(),
                    Flag: Some("YELLOW".into()),
                    Scope: Some("Sector".into()),
                    Sector: Some(11),
                },
                RawRaceControlMessage {
                    Utc: "time2".into(),
                    Category: "Flag".into(),
                    Message: "YELLOW IN TRACK SECTOR 12".into(),
                    Flag: Some("DOUBLE YELLOW".into()),
                    Scope: Some("Sector".into()),
                    Sector: Some(12),
                },
            ],
        };
        let status = compute_circuit_status(Some(&rcm));
        if let CircuitStatus::Yellow(CircuitScope::Sectors(sectors)) = status {
            assert_eq!(sectors, vec![11, 12]);
        } else {
            panic!("Expected Yellow with sectors 11 and 12");
        }
    }
}
