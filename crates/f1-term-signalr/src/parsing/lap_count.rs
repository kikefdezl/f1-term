use f1_term_core::laps::Laps;
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct LapCountPayload {
    CurrentLap: u8,
    TotalLaps: u8,
}

impl From<LapCountPayload> for Laps {
    fn from(value: LapCountPayload) -> Self {
        Laps {
            current: value.CurrentLap,
            total: value.TotalLaps,
        }
    }
}

pub fn parse_lap_count(val: &Value) -> Result<Laps> {
    match val {
        Value::Object(_) => match LapCountPayload::deserialize(val) {
            Ok(lc) => Ok(Laps::from(lc)),
            Err(e) => Err(format!("Failed to parse LapCountPayload: {}", e).into()),
        },
        _ => Err("LapCount value is not a JSON object".into()),
    }
}
