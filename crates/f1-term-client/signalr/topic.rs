use std::fmt::Display;

pub enum Topic {
    Heartbeat,
    ExtrapolatedClock,
    TimingStats,
    TimingAppData,
    WeatherData,
    TrackStatus,
    DriverList,
    RaceControlMessages,
    SessionInfo,
    SessionData,
    LapCount,
    TimingData,
    TeamRadio,
    CarDataZ,
    PositionZ,
    ChampionshipPrediction,
    PitLaneTimeCollection,
    PitStopSeries,
}

impl Topic {
    pub fn all() -> Vec<Topic> {
        vec![
            Topic::Heartbeat,
            Topic::ExtrapolatedClock,
            Topic::TimingStats,
            Topic::TimingAppData,
            Topic::WeatherData,
            Topic::TrackStatus,
            Topic::DriverList,
            Topic::RaceControlMessages,
            Topic::SessionInfo,
            Topic::SessionData,
            Topic::LapCount,
            Topic::TimingData,
            Topic::TeamRadio,
            Topic::CarDataZ,
            Topic::PositionZ,
            Topic::ChampionshipPrediction,
            Topic::PitLaneTimeCollection,
            Topic::PitStopSeries,
        ]
    }
}

impl Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Topic::Heartbeat => write!(f, "Heartbeat"),
            Topic::ExtrapolatedClock => write!(f, "ExtrapolatedClock"),
            Topic::TimingStats => write!(f, "TimingStats"),
            Topic::TimingAppData => write!(f, "TimingAppData"),
            Topic::WeatherData => write!(f, "WeatherData"),
            Topic::TrackStatus => write!(f, "TrackStatus"),
            Topic::DriverList => write!(f, "DriverList"),
            Topic::RaceControlMessages => write!(f, "RaceControlMessages"),
            Topic::SessionInfo => write!(f, "SessionInfo"),
            Topic::SessionData => write!(f, "SessionData"),
            Topic::LapCount => write!(f, "LapCount"),
            Topic::TimingData => write!(f, "TimingData"),
            Topic::TeamRadio => write!(f, "TeamRadio"),
            Topic::CarDataZ => write!(f, "CarData.z"),
            Topic::PositionZ => write!(f, "Position.z"),
            Topic::ChampionshipPrediction => write!(f, "ChampionshipPrediction"),
            Topic::PitLaneTimeCollection => write!(f, "PitLaneTimeCollection"),
            Topic::PitStopSeries => write!(f, "PitStopSeries"),
        }
    }
}
