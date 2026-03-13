pub mod driver_list;
pub mod extrapolated_clock;
pub mod lap_count;
pub mod race_control_messages;
pub mod session_data;
pub mod session_info;
pub mod stints;
pub mod timing_data;
pub mod track_status;
pub mod weather_data;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
