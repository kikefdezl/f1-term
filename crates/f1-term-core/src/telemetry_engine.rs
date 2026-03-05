use std::sync::{Arc, RwLock};

use chrono::Datelike;

use super::{
    circuit::{CircuitLayout, CircuitLayoutProvider},
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
            let fetched_layout = self.update_circuit_layout(&updates).await;

            let mut state_lock = self.state.write().unwrap();
            let mut state_changed = false;

            for update in updates {
                if state_lock.apply(update) {
                    state_changed = true;
                }
            }

            if let Some(layout) = fetched_layout {
                if let Some(info_mut) = &mut state_lock.info {
                    info_mut.meeting.circuit.layout = Some(layout);
                }
                state_changed = true;
            }

            if state_changed {
                state_lock.update_version += 1;
            }
        }
    }

    /// A hook to fetch the CircuitLayout from the circuit provider after the telemetry
    /// provider has fetched the circuit key.
    /// Only updates and returns a CircuitLayout if the circuit key returned by the
    /// telemetry provider has changed (to avoid pinging the circuit layout provider on every
    /// update).
    /// Returns None if no change.
    async fn update_circuit_layout(
        &mut self,
        updates: &[TelemetryUpdate],
    ) -> Option<CircuitLayout> {
        let mut fetched_layout = None;
        for update in updates {
            if let TelemetryUpdate::SessionInfo(info) = update {
                let circuit_key = info.meeting.circuit.key;
                if circuit_key != self._cached_circuit_key {
                    self._cached_circuit_key = circuit_key;
                    let year = info.start_date.year() as u32;
                    match self.circuit_provider.fetch(circuit_key, year).await {
                        Ok(layout) => fetched_layout = Some(layout),
                        Err(e) => log::warn!(
                            "Failed to fetch circuit layout for key {}: {}",
                            circuit_key,
                            e
                        ),
                    }
                }
            }
        }
        fetched_layout
    }
}
