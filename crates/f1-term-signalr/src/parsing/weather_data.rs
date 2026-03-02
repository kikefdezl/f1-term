use f1_term_core::weather::{Weather, Wind, WindDirection};
use serde::Deserialize;
use serde_json::Value;

use super::Result;

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct WeatherDataPayload {
    AirTemp: String,
    Humidity: String,
    Pressure: String,
    Rainfall: String,
    TrackTemp: String,
    WindDirection: String,
    WindSpeed: String,
}

impl TryFrom<WeatherDataPayload> for Weather {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: WeatherDataPayload) -> Result<Self> {
        Ok(Weather {
            air_temperature: value.AirTemp.parse()?,
            track_temperature: value.TrackTemp.parse()?,
            humidity: value.Humidity.parse()?,
            pressure: value.Pressure.parse()?,
            rainfall: value.Rainfall.parse()?,
            wind: Wind {
                direction: WindDirection {
                    value: value.WindDirection.parse()?,
                },
                speed: value.WindSpeed.parse()?,
            },
        })
    }
}
pub fn parse_weather_data(val: &Value) -> Result<Weather> {
    let payload: WeatherDataPayload = WeatherDataPayload::deserialize(val)?;
    Weather::try_from(payload)
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

        let result = parse_weather_data(&json_data).expect("Failed to parse weather data");

        assert_eq!(result.air_temperature, 23.5);
        assert_eq!(result.humidity, 34.3);
        assert_eq!(result.pressure, 1018.2);
        assert_eq!(result.rainfall, 0.0);
        assert_eq!(result.track_temperature, 25.8);
        assert_eq!(result.wind.direction.value, 353.0);
        assert_eq!(result.wind.speed, 2.0);
    }
}
