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
