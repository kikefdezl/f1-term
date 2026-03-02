use std::sync::{Arc, RwLock};

use chrono::Datelike;

use super::{
    circuit::CircuitLayoutProvider,
    telemetry_provider::{TelemetryProvider, TelemetryUpdate},
    telemetry_state::TelemetryState,
};

pub struct TelemetryEngine<T: TelemetryProvider, C: CircuitLayoutProvider> {
    pub state: Arc<RwLock<TelemetryState>>,
    pub telemetry_provider: T,
    pub circuit_provider: C,
    pub _cached_circuit_key: u32,
}

impl<T: TelemetryProvider, C: CircuitLayoutProvider> TelemetryEngine<T, C> {
    pub fn new(f1_client: T, circuit_layout_provider: C) -> Self {
        TelemetryEngine {
            state: Arc::new(RwLock::new(TelemetryState::default())),
            telemetry_provider: f1_client,
            circuit_provider: circuit_layout_provider,
            _cached_circuit_key: u32::MAX,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.telemetry_provider.connect().await
    }

    pub fn get_state(&self) -> Arc<RwLock<TelemetryState>> {
        Arc::clone(&self.state)
    }

    pub async fn run(&mut self) {
        while let Some(updates) = self.telemetry_provider.next_updates().await {
            let mut fetched_layout = None;
            for update in &updates {
                if let TelemetryUpdate::SessionInfo(info) = update {
                    let circuit_key = info.meeting.circuit.key;
                    if circuit_key != self._cached_circuit_key {
                        let year = info.start_date.year() as u32;
                        if let Ok(layout) = self.circuit_provider.fetch(circuit_key, year).await {
                            fetched_layout = Some((circuit_key, layout));
                        }
                    }
                }
            }

            let mut state_lock = self.state.write().unwrap();
            let mut state_changed = false;

            for update in updates {
                if state_lock.apply(update) {
                    state_changed = true;
                }
            }

            if let Some((circuit_key, layout)) = fetched_layout {
                if let Some(info_mut) = &mut state_lock.info {
                    info_mut.meeting.circuit.layout = Some(layout);
                }
                self._cached_circuit_key = circuit_key;
                state_changed = true;
            }

            if state_changed {
                state_lock.update_version += 1;
            }
        }
    }
}
