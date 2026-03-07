use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct RawWeatherData {
    pub AirTemp: String,
    pub Humidity: String,
    pub Pressure: String,
    pub Rainfall: String,
    pub TrackTemp: String,
    pub WindDirection: String,
    pub WindSpeed: String,
}

pub fn parse_raw_weather_data(val: &Value) -> Result<RawWeatherData> {
    let payload = RawWeatherData::deserialize(val)?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parse_weather_data_from_json() {
        let json_data = json!({
            "AirTemp": "23.5",
            "Humidity": "34.3",
            "Pressure": "1018.2",
            "Rainfall": "0",
            "TrackTemp": "25.8",
            "WindDirection": "353",
            "WindSpeed": "2.0",
            "_kf": true
        });

        let raw = parse_raw_weather_data(&json_data).expect("Failed to parse raw weather data");

        assert_eq!(raw.AirTemp, "23.5");
        assert_eq!(raw.Humidity, "34.3");
        assert_eq!(raw.Pressure, "1018.2");
        assert_eq!(raw.Rainfall, "0");
        assert_eq!(raw.TrackTemp, "25.8");
        assert_eq!(raw.WindDirection, "353");
        assert_eq!(raw.WindSpeed, "2.0");
    }
}
