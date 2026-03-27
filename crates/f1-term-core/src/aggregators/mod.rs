pub mod layout_fetch;
pub mod sector_count;

use crate::telemetry_engine::EngineTask;
use crate::telemetry_provider::TelemetryUpdate;
use crate::telemetry_state::TelemetryState;

pub trait TelemetryAggregator: Send {
    fn process(
        &self,
        current_state: &TelemetryState,
        update: &mut TelemetryUpdate,
    ) -> Vec<EngineTask>;
}
