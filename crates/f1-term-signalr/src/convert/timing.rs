use std::collections::HashMap;

use f1_term_core::{
    driver::DriverNumber,
    timing::{Lap, LiveTiming, Sector, Segment, SegmentStatus, Speed, Speeds, TimeDiffs},
};
use log::{info, warn};

use crate::parsing::timing_data::{
    RawSector, RawSegment, RawSpeed, RawSpeeds, RawStats, RawTimingData,
};

impl From<&RawSegment> for Segment {
    fn from(p: &RawSegment) -> Self {
        let status = match p.Status {
            0 => SegmentStatus::None,
            2048 => SegmentStatus::Normal,
            2049 => SegmentStatus::PersonalFastest,
            2050 => SegmentStatus::Unknown,
            2051 => SegmentStatus::OverallFastest,
            2052 => SegmentStatus::Aborted,
            2064 => SegmentStatus::InPit,
            other => {
                warn!("Unknown SegmentStatus value {}!", other);
                SegmentStatus::None
            }
        };
        Segment { status }
    }
}

impl From<&RawSector> for Sector {
    fn from(p: &RawSector) -> Self {
        let value = Some(p.Value.clone()).filter(|s| !s.is_empty());
        Sector {
            overall_fastest: p.OverallFastest,
            personal_fastest: p.PersonalFastest,
            segments: p.Segments.iter().map(Into::into).collect(),
            status: p.Status,
            stopped: p.Stopped,
            value,
            previous_value: p.PreviousValue.clone(),
        }
    }
}

impl From<&RawSpeed> for Speed {
    fn from(p: &RawSpeed) -> Self {
        Speed {
            overall_fastest: p.OverallFastest,
            personal_fastest: p.PersonalFastest,
            status: p.Status,
            value: p.Value.clone(),
        }
    }
}

impl From<&RawSpeeds> for Speeds {
    fn from(p: &RawSpeeds) -> Self {
        Speeds {
            fl: (&p.FL).into(),
            i1: (&p.I1).into(),
            i2: (&p.I2).into(),
            st: (&p.ST).into(),
        }
    }
}

impl From<&RawStats> for TimeDiffs {
    fn from(value: &RawStats) -> Self {
        let to_fastest = Some(value.TimeDiffToFastest.clone()).filter(|s| !s.is_empty());
        let to_position_ahead =
            Some(value.TimeDifftoPositionAhead.clone()).filter(|s| !s.is_empty());
        Self {
            to_fastest,
            to_position_ahead,
        }
    }
}

impl TryFrom<&RawTimingData> for LiveTiming {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: &RawTimingData) -> Result<Self, Self::Error> {
        let driver_number = DriverNumber {
            value: payload.RacingNumber.parse()?,
        };

        // API returns empty strings, we convert those to None
        let best_lap_time = Some(payload.BestLapTime.Value.clone()).filter(|s| !s.is_empty());
        let last_lap_time = Some(payload.LastLapTime.Value.clone()).filter(|s| !s.is_empty());
        let time_diff_to_fastest =
            Some(payload.TimeDiffToFastest.clone()).filter(|s| !s.is_empty());
        let time_diff_to_position_ahead =
            Some(payload.TimeDiffToPositionAhead.clone()).filter(|s| !s.is_empty());

        let last_lap = Lap {
            overall_fastest: payload.LastLapTime.OverallFastest,
            personal_fastest: payload.LastLapTime.PersonalFastest,
            status: payload.LastLapTime.Status,
            time: last_lap_time,
            sectors: payload.Sectors.iter().map(Into::into).collect(),
            show_position: payload.ShowPosition,
            speeds: (&payload.Speeds).into(),
        };

        let time_diffs = TimeDiffs {
            to_fastest: time_diff_to_fastest,
            to_position_ahead: time_diff_to_position_ahead,
        };

        let quali_stats = payload
            .Stats
            .as_ref()
            .map(|s| s.iter().map(|stat| stat.into()).collect::<Vec<TimeDiffs>>());

        let lap_data = f1_term_core::timing::LapData {
            best_lap_time,
            last_lap,
            number_of_laps: payload.NumberOfLaps,
        };

        let pit_data = f1_term_core::timing::PitData {
            in_pit: payload.InPit,
            pit_out: payload.PitOut,
            number_of_pit_stops: payload.NumberOfPitStops,
        };

        let quali_stats =
            if payload.Cutoff.is_some() || payload.KnockedOut.is_some() || quali_stats.is_some() {
                Some(f1_term_core::timing::QualiStats {
                    cutoff: payload.Cutoff,
                    knocked_out: payload.KnockedOut,
                    diffs: quali_stats,
                })
            } else {
                None
            };

        Ok(LiveTiming {
            driver_number,
            position: payload.Position.parse().unwrap_or(0),
            status: payload.Status,
            retired: payload.Retired,
            stopped: payload.Stopped,
            time_diffs,
            lap_data,
            pit_data,
            quali_stats,
        })
    }
}

pub fn convert_timing_data(
    raw_timing_data: &HashMap<String, RawTimingData>,
) -> HashMap<DriverNumber, LiveTiming> {
    let mut timing_data: HashMap<DriverNumber, LiveTiming> = HashMap::new();

    for (num_str, payload) in raw_timing_data {
        let Ok(number) = num_str.parse::<u8>() else {
            warn!("Failed to parse timing data line {}", num_str);
            continue;
        };

        let driver_number = DriverNumber { value: number };

        match LiveTiming::try_from(payload) {
            Ok(lt) => {
                timing_data.insert(driver_number, lt);
            }
            Err(e) => {
                info!(
                    "Failed to convert live timing payload for {}: {}",
                    num_str, e
                );
            }
        }
    }

    timing_data
}
