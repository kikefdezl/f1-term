use std::collections::HashMap;

use super::TelemetryAggregator;
use crate::driver_position::DriverPosition;
use crate::telemetry_engine::EngineTask;
use crate::telemetry_provider::TelemetryUpdate;
use crate::telemetry_state::TelemetryState;

pub struct PositionAggregator;

impl TelemetryAggregator for PositionAggregator {
    fn process(
        &self,
        current_state: &TelemetryState,
        update: &mut TelemetryUpdate,
    ) -> Vec<EngineTask> {
        if let Some(timing_data) = &update.timing_data {
            for (driver_number, driver) in &current_state.drivers {
                if driver.position.is_none() {
                    let position = DriverPosition::default(); // TODO
                    update
                        .drivers
                        .get_or_insert_with(HashMap::new)
                        .insert(driver_number.clone(), driver.clone().with_position(position));
                }
            }
        }
        vec![]
    }
}
