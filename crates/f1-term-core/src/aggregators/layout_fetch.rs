use super::TelemetryAggregator;
use crate::telemetry_engine::EngineTask;
use crate::telemetry_provider::TelemetryUpdate;
use crate::telemetry_state::TelemetryState;

pub struct LayoutFetchAggregator;

impl TelemetryAggregator for LayoutFetchAggregator {
    fn process(
        &self,
        current_state: &TelemetryState,
        update: &mut TelemetryUpdate,
    ) -> Vec<EngineTask> {
        let mut tasks = vec![];
        if let Some(circuit_update) = &update.circuit {
            if let Some(prev_circuit) = &current_state.circuit {
                if circuit_update.key != prev_circuit.key {
                    tasks.push(EngineTask::FetchCircuitLayout);
                }
            } else {
                tasks.push(EngineTask::FetchCircuitLayout);
            }
        }
        tasks
    }
}
