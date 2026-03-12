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
