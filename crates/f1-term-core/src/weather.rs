use std::fmt::Display;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Weather {
    /// Air temperature in Celsius
    pub air_temperature: f32,
    /// Track temperature in Celsius
    pub track_temperature: f32,
    /// Humidity in percentage
    pub humidity: f32,
    /// Pressure in millibars
    pub pressure: f32,
    /// Rainfall in percentage
    pub rainfall: f32,
    pub wind: Wind,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Wind {
    /// Wind speed in m/s
    pub speed: f32,
    pub direction: WindDirection,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct WindDirection {
    /// Value in degrees
    pub value: f32,
}

impl WindDirection {
    pub fn to_direction(&self) -> Direction {
        if self.value <= 22.5 || self.value >= 337.5 {
            Direction::N
        } else if self.value <= 67.5 {
            Direction::NE
        } else if self.value <= 112.5 {
            Direction::E
        } else if self.value <= 157.5 {
            Direction::SE
        } else if self.value <= 202.5 {
            Direction::S
        } else if self.value <= 247.5 {
            Direction::SW
        } else if self.value <= 292.5 {
            Direction::W
        } else {
            Direction::NW
        }
    }
}

pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::N => write!(f, "N"),
            Self::NE => write!(f, "NE"),
            Self::E => write!(f, "E"),
            Self::SE => write!(f, "SE"),
            Self::S => write!(f, "S"),
            Self::SW => write!(f, "SW"),
            Self::W => write!(f, "W"),
            Self::NW => write!(f, "NW"),
        }
    }
}
