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
    pub circuit_provider: Arc<C>,
    pub _cached_circuit_key: u32,
}

impl<T: TelemetryProvider, C: CircuitLayoutProvider + 'static> TelemetryEngine<T, C> {
    pub fn new(f1_client: T, circuit_layout_provider: C) -> Self {
        TelemetryEngine {
            state: Arc::new(RwLock::new(TelemetryState::default())),
            telemetry_provider: f1_client,
            circuit_provider: Arc::new(circuit_layout_provider),
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
            self.check_and_fetch_circuit_layout(&updates);
            self.apply_updates(updates);
        }
    }

    fn check_and_fetch_circuit_layout(&mut self, updates: &[TelemetryUpdate]) {
        for update in updates {
            if let TelemetryUpdate::SessionInfo(info) = update {
                let circuit_key = info.meeting.circuit.key;
                if circuit_key != self._cached_circuit_key {
                    self._cached_circuit_key = circuit_key;
                    let year = info.start_date.year() as u32;
                    self.spawn_layout_fetch(circuit_key, year);
                }
            }
        }
    }

    fn spawn_layout_fetch(&self, circuit_key: u32, year: u32) {
        let provider = Arc::clone(&self.circuit_provider);
        let state_arc = Arc::clone(&self.state);

        tokio::spawn(async move {
            match provider.fetch(circuit_key, year).await {
                Ok(layout) => {
                    let mut state_lock = state_arc.write().unwrap();
                    if let Some(info_mut) = &mut state_lock.info {
                        if info_mut.meeting.circuit.key == circuit_key {
                            info_mut.meeting.circuit.layout = Some(layout);
                            state_lock.update_version += 1;
                        } else {
                            log::warn!(
                                "Circuit key changed while fetching layout for {}, discarding.",
                                circuit_key
                            );
                        }
                    }
                }
                Err(e) => log::warn!(
                    "Failed to fetch circuit layout for key {}: {}",
                    circuit_key,
                    e
                ),
            }
        });
    }

    fn apply_updates(&self, updates: Vec<TelemetryUpdate>) {
        let mut state_lock = self.state.write().unwrap();
        let mut state_changed = false;

        for update in updates {
            if state_lock.apply(update) {
                state_changed = true;
            }
        }

        if state_changed {
            state_lock.update_version += 1;
        }
    }
}
