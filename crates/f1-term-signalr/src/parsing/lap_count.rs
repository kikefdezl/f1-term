use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawLapCount {
    pub CurrentLap: u8,
    pub TotalLaps: u8,
}

pub fn parse_raw_lap_count(val: &Value) -> Result<RawLapCount> {
    match val {
        Value::Object(_) => match RawLapCount::deserialize(val) {
            Ok(lc) => Ok(lc),
            Err(e) => Err(format!("Failed to parse RawLapCount: {}", e).into()),
        },
        _ => Err("LapCount value is not a JSON object".into()),
    }
}
