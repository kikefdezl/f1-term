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
            update.circuit_layout = Some(layout.interpolate_mini_sectors(count));
        }
        vec![]
    }
}
