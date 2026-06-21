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
            Direction::North
        } else if self.value <= 67.5 {
            Direction::NorthEast
        } else if self.value <= 112.5 {
            Direction::East
        } else if self.value <= 157.5 {
            Direction::SouthEast
        } else if self.value <= 202.5 {
            Direction::South
        } else if self.value <= 247.5 {
            Direction::SouthWest
        } else if self.value <= 292.5 {
            Direction::West
        } else {
            Direction::NorthWest
        }
    }
}

pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::North => write!(f, "N"),
            Self::NorthEast => write!(f, "NE"),
            Self::East => write!(f, "E"),
            Self::SouthEast => write!(f, "SE"),
            Self::South => write!(f, "S"),
            Self::SouthWest => write!(f, "SW"),
            Self::West => write!(f, "W"),
            Self::NorthWest => write!(f, "NW"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_to_direction() {
        fn assert(value: f32, expected: &str) {
            assert_eq!(WindDirection { value }.to_direction().to_string(), expected);
        }
        assert(10.0, "N");
        assert(22.5, "N");
        assert(22.6, "NE");
        assert(45.0, "NE");
        assert(67.5, "NE");
        assert(90.0, "E");
        assert(135.0, "SE");
        assert(180.0, "S");
        assert(225.0, "SW");
        assert(270.0, "W");
        assert(315.0, "NW");
        assert(337.5, "N");
        assert(350.0, "N");
        assert(360.0, "N");
    }
}
