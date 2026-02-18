use super::driver::DriverNumber;

struct LiveTiming {
    driver_number: DriverNumber,
    best_lap_time: String,
    in_pit: bool,
    pit_out: bool,
    last_lap: LastLap,
    position: u8,
    retired: bool,
    status: u8,
    stopped: bool,
    time_diff_to_fastest: String,
    time_diff_to_position_ahead: String,
}

struct LastLap {
    overall_fastest: bool,
    personal_fastest: bool,
    status: u8,
    time: String,
    sectors: Vec<Sector>,
    show_position: bool,
    speeds: Speeds,
}

struct Sector {
    overall_fastest: bool,
    personal_fastest: bool,
    segments: Vec<Segment>,
    status: u8,
    stopped: bool,
    value: String,
}

struct Segment {
    status: u8,
}

struct Speeds {
    fl: Speed,
    i1: Speed,
    i2: Speed,
    st: Speed,
}

struct Speed {
    overall_fastest: bool,
    personal_fastest: bool,
    status: u8,
    value: String,
}
