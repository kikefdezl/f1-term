use crate::aggregators::TelemetryAggregator;
use crate::telemetry_engine::EngineTask;
use crate::telemetry_provider::TelemetryUpdate;
use crate::telemetry_state::TelemetryState;

pub struct SectorCountAggregator;

impl TelemetryAggregator for SectorCountAggregator {
    fn process(
        &self,
        current_state: &TelemetryState,
        update: &mut TelemetryUpdate,
    ) -> Vec<EngineTask> {
        if let Some(circuit) = &current_state.circuit
            && let Some(layout) = &circuit.layout
            && layout.mini_sectors.is_none()
            && let Some(timing_data) = &update.timing_data
            && let Some(live_timing) = timing_data.values().next()
        {
            let count = live_timing
                .lap_data
                .last_lap
                .sectors
                .iter()
                .map(|s| s.segments.len())
                .sum();
            if count > 0 {
                update.circuit_layout = Some(layout.interpolate_mini_sectors(count));
            }
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::circuit::{Circuit, CircuitLayout};
    use crate::driver::DriverNumber;
    use crate::timing::LiveTiming;

    // If the app is running when the session changes, the segment count for the drivers is
    // briefly zero, leading to a division by zero error in the interpolate_mini_sectors() function.
    #[test]
    fn test_count_zero() {
        let current_state = TelemetryState {
            circuit: Some(Circuit {
                layout: Some(CircuitLayout::default()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let timing_data = HashMap::from([(DriverNumber { value: 0 }, LiveTiming::default())]);
        let mut update = TelemetryUpdate {
            timing_data: Some(timing_data),
            ..Default::default()
        };
        let aggregator = SectorCountAggregator;
        let _ = aggregator.process(&current_state, &mut update);
        assert!(update.circuit_layout.is_none());
    }
}
