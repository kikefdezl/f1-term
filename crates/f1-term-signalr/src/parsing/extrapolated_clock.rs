use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct RawExtrapolatedClock {
    pub Extrapolating: bool,
    pub Remaining: String,
    pub Utc: String,
}

pub fn parse_extrapolated_clock(val: &Value) -> Result<RawExtrapolatedClock> {
    match val {
        Value::Object(_) => match RawExtrapolatedClock::deserialize(val) {
            Ok(ec) => Ok(ec),
            Err(e) => Err(format!("Failed to parse RawExtrapolatedClock: {}", e).into()),
        },
        _ => Err("RawExtrapolatedClock value is not a JSON object".into()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_extrapolated_clock() {
        let json_data = json!({
            "Extrapolating": true,
            "Remaining": "00:22:01",
            "Utc": "2026-03-13T03:39:11.3375981Z",
            "_kf": true
        });

        let result =
            parse_extrapolated_clock(&json_data).expect("Failed to parse raw extrapolated clock");

        assert!(result.Extrapolating);
        assert_eq!(result.Remaining, "00:22:01");
    }
}
