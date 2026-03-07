use f1_term_core::weather::{Weather, Wind, WindDirection};

use crate::parsing::weather_data::RawWeatherData;

impl TryFrom<&RawWeatherData> for Weather {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &RawWeatherData) -> Result<Self, Self::Error> {
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

pub fn convert_weather_data(raw: &RawWeatherData) -> Result<Weather, Box<dyn std::error::Error>> {
    Weather::try_from(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_weather_data() {
        let raw = RawWeatherData {
            AirTemp: "23.5".to_string(),
            Humidity: "34.3".to_string(),
            Pressure: "1018.2".to_string(),
            Rainfall: "0".to_string(),
            TrackTemp: "25.8".to_string(),
            WindDirection: "353".to_string(),
            WindSpeed: "2.0".to_string(),
        };

        let result = convert_weather_data(&raw).expect("Failed to convert weather data");

        assert_eq!(result.air_temperature, 23.5);
        assert_eq!(result.humidity, 34.3);
        assert_eq!(result.pressure, 1018.2);
        assert_eq!(result.rainfall, 0.0);
        assert_eq!(result.track_temperature, 25.8);
        assert_eq!(result.wind.direction.value, 353.0);
        assert_eq!(result.wind.speed, 2.0);
    }
}
