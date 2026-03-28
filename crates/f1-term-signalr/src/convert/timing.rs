use std::collections::HashMap;

use f1_term_core::driver::DriverNumber;
use f1_term_core::lap_time::LapTime;
use f1_term_core::timing::{
    BestLap, LapData, LastLap, LiveTiming, Sector, Segment, SegmentStatus, Speed, Speeds, TimeDiffs,
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

// TODO: This should be a private/hidden function since we can't really infer the status
// of the best lap (whether it's overall fastest or not). This forces us to then loop
// through all the drivers to determine the fastest, so this function is not a self-contained.
impl TryFrom<&RawTimingData> for LiveTiming {
    type Error = Box<dyn std::error::Error>;

    fn try_from(payload: &RawTimingData) -> Result<Self, Self::Error> {
        let driver_number = DriverNumber {
            value: payload.RacingNumber.parse()?,
        };

        // API returns empty strings, we convert those to None via filter
        let best_lap_time = Some(payload.BestLapTime.Value.clone())
            .filter(|s| !s.is_empty())
            .map(|blp| LapTime::try_from(blp.as_str()))
            .transpose()?;
        let last_lap_time = Some(payload.LastLapTime.Value.clone())
            .filter(|s| !s.is_empty())
            .map(|llt| LapTime::try_from(llt.as_str()))
            .transpose()?;

        let time_diff_to_fastest = payload
            .TimeDiffToFastest
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| payload.GapToLeader.clone().filter(|s| !s.is_empty()));
        let time_diff_to_position_ahead = payload
            .TimeDiffToPositionAhead
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                payload
                    .IntervalToPositionAhead
                    .as_ref()
                    .map(|i| i.Value.clone())
                    .filter(|s| !s.is_empty())
            });

        let last_lap = LastLap {
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

        let mut q1_diffs = None;
        let mut q2_diffs = None;
        let mut q3_diffs = None;

        if let Some(stats) = &payload.Stats {
            if let Some(stat) = stats.first() {
                q1_diffs = Some(stat.into());
            }
            if let Some(stat) = stats.get(1) {
                q2_diffs = Some(stat.into());
            }
            if let Some(stat) = stats.get(2) {
                q3_diffs = Some(stat.into());
            }
        }

        let lap_data = LapData {
            best_lap: BestLap {
                time: best_lap_time,
                // NOTE: This cannot be checked from the raw payload directly so it must be
                // compared with all other drivers after and set to true
                overall_fastest: false,
            },
            last_lap,
            number_of_laps: payload.NumberOfLaps,
        };

        let pit_data = f1_term_core::timing::PitData {
            in_pit: payload.InPit,
            pit_out: payload.PitOut,
            number_of_pit_stops: payload.NumberOfPitStops,
        };

        let quali_stats = if payload.Cutoff.is_some()
            || payload.KnockedOut.is_some()
            || payload.Stats.is_some()
        {
            Some(f1_term_core::timing::QualiStats {
                cutoff: payload.Cutoff,
                knocked_out: payload.KnockedOut,
                q1_diffs,
                q2_diffs,
                q3_diffs,
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

    let mut fastest_time = LapTime {
        minutes: u32::MAX,
        ..Default::default()
    };

    for (num_str, payload) in raw_timing_data {
        let Ok(number) = num_str.parse::<u8>() else {
            warn!("Failed to parse timing data line {}", num_str);
            continue;
        };

        let driver_number = DriverNumber { value: number };

        match LiveTiming::try_from(payload) {
            Ok(lt) => {
                if let Some(time) = &lt.lap_data.best_lap.time
                    && *time < fastest_time
                {
                    fastest_time = time.clone()
                }
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

    for timing in timing_data.values_mut() {
        if let Some(time) = &timing.lap_data.best_lap.time
            && *time == fastest_time
        {
            timing.lap_data.best_lap.overall_fastest = true;
        }
    }

    timing_data
}

#[cfg(test)]
mod tests {
    use f1_term_core::driver::DriverNumber;

    use super::*;
    use crate::parsing::timing_data::{
        RawBestLapTime, RawLastLapTime, RawSector, RawSegment, RawSpeed, RawSpeeds, RawTimingData,
    };

    #[test]
    fn test_convert_timing_data() {
        let mut raw = HashMap::new();
        raw.insert(
            "1".to_string(),
            RawTimingData {
                RacingNumber: "1".to_string(),
                BestLapTime: RawBestLapTime {
                    Value: "1:23.456".to_string(),
                },
                InPit: false,
                PitOut: false,
                LastLapTime: RawLastLapTime {
                    OverallFastest: false,
                    PersonalFastest: true,
                    Status: 0,
                    Value: "1:24.000".to_string(),
                },
                Position: "1".to_string(),
                Retired: false,
                Status: 0,
                Stopped: false,
                TimeDiffToFastest: Some("".to_string()),
                TimeDiffToPositionAhead: Some("".to_string()),
                GapToLeader: None,
                IntervalToPositionAhead: None,
                Sectors: vec![RawSector {
                    OverallFastest: false,
                    PersonalFastest: true,
                    Segments: vec![RawSegment { Status: 0 }],
                    Status: 0,
                    Stopped: false,
                    Value: "25.1".to_string(),
                    PreviousValue: Some("25.6".to_string()),
                }],
                ShowPosition: true,
                Speeds: RawSpeeds {
                    FL: RawSpeed {
                        OverallFastest: false,
                        PersonalFastest: false,
                        Status: 0,
                        Value: "320".to_string(),
                    },
                    I1: RawSpeed::default(),
                    I2: RawSpeed::default(),
                    ST: RawSpeed::default(),
                },
                Cutoff: None,
                KnockedOut: None,
                NumberOfLaps: None,
                NumberOfPitStops: None,
                Stats: None,
            },
        );

        let data = convert_timing_data(&raw);
        assert_eq!(data.len(), 1);

        let driver_number = DriverNumber { value: 1 };
        let timing = data.get(&driver_number).unwrap();

        assert_eq!(timing.position, 1);
        assert_eq!(
            timing.lap_data.best_lap.time,
            Some(LapTime::new(1, 23, 456))
        );
        assert!(timing.lap_data.best_lap.overall_fastest);
        assert_eq!(timing.lap_data.last_lap.time, Some(LapTime::new(1, 24, 0)));
        assert!(timing.lap_data.last_lap.personal_fastest);

        assert_eq!(timing.lap_data.last_lap.sectors.len(), 1);
        assert_eq!(
            timing.lap_data.last_lap.sectors[0].value.as_deref(),
            Some("25.1")
        );
        assert!(timing.lap_data.last_lap.sectors[0].personal_fastest);

        assert_eq!(timing.lap_data.last_lap.speeds.fl.value, "320");
    }

    #[test]
    fn test_timing_data_missing_fields() {
        let mut raw = HashMap::new();
        raw.insert(
            "44".to_string(),
            RawTimingData {
                RacingNumber: "44".to_string(),
                Position: "1".to_string(),
                InPit: false,
                PitOut: false,
                Retired: false,
                Status: 0,
                Stopped: false,
                ShowPosition: true,
                Sectors: vec![],
                Speeds: RawSpeeds {
                    FL: RawSpeed {
                        Status: 0,
                        ..Default::default()
                    },
                    I1: RawSpeed {
                        Status: 0,
                        ..Default::default()
                    },
                    I2: RawSpeed {
                        Status: 0,
                        ..Default::default()
                    },
                    ST: RawSpeed {
                        Status: 0,
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
        );

        let map = convert_timing_data(&raw);
        let driver_timing = map.get(&DriverNumber { value: 44 }).unwrap();

        assert_eq!(driver_timing.lap_data.best_lap.time, None);
        assert!(!driver_timing.lap_data.best_lap.overall_fastest);
        assert_eq!(driver_timing.lap_data.last_lap.time, None);
        assert_eq!(driver_timing.time_diffs.to_fastest, None);
    }
}
