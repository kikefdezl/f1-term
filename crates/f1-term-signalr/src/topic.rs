use std::fmt::Display;

// Unused topics for reference:
// - ArchiveStatus
// - AudioStreams
// - CarDataZ (CarData.z)
// - ChampionshipPrediction
// - ContentStreams
// - CurrentTyres
// - DriverRaceInfo
// - DriverTracker
// - Heartbeat
// - LapSeries
// - OvertakeSeries
// - PitLaneTimeCollection
// - PitStop
// - PitStopSeries
// - PositionZ (Position.z)
// - SessionStatus
// - TeamRadio
// - TimingDataF1
// - TimingStats
// - TlaRcm
// - TopThree
// - TyreStintSeries
// - WeatherDataSeries
#[derive(PartialEq)]
pub enum Topic {
    DriverList,
    ExtrapolatedClock,
    LapCount,
    RaceControlMessages,
    SessionData,
    SessionInfo,
    TimingAppData,
    TimingData,
    TrackStatus,
    WeatherData,
}

impl Topic {
    pub fn all() -> Vec<Topic> {
        vec![
            Topic::DriverList,
            Topic::ExtrapolatedClock,
            Topic::LapCount,
            Topic::RaceControlMessages,
            Topic::SessionData,
            Topic::SessionInfo,
            Topic::TimingAppData,
            Topic::TimingData,
            Topic::TrackStatus,
            Topic::WeatherData,
        ]
    }
}

impl Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Topic::DriverList => write!(f, "DriverList"),
            Topic::ExtrapolatedClock => write!(f, "ExtrapolatedClock"),
            Topic::LapCount => write!(f, "LapCount"),
            Topic::RaceControlMessages => write!(f, "RaceControlMessages"),
            Topic::SessionData => write!(f, "SessionData"),
            Topic::SessionInfo => write!(f, "SessionInfo"),
            Topic::TimingAppData => write!(f, "TimingAppData"),
            Topic::TimingData => write!(f, "TimingData"),
            Topic::TrackStatus => write!(f, "TrackStatus"),
            Topic::WeatherData => write!(f, "WeatherData"),
        }
    }
}

impl TryFrom<&str> for Topic {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "DriverList" => Ok(Topic::DriverList),
            "ExtrapolatedClock" => Ok(Topic::ExtrapolatedClock),
            "LapCount" => Ok(Topic::LapCount),
            "RaceControlMessages" => Ok(Topic::RaceControlMessages),
            "SessionData" => Ok(Topic::SessionData),
            "SessionInfo" => Ok(Topic::SessionInfo),
            "TimingAppData" => Ok(Topic::TimingAppData),
            "TimingData" => Ok(Topic::TimingData),
            "TrackStatus" => Ok(Topic::TrackStatus),
            "WeatherData" => Ok(Topic::WeatherData),
            _ => Err(format!("Unknown topic {}", value).into()),
        }
    }
}
