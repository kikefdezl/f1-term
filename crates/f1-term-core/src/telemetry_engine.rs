use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use log::info;
use tokio::time::sleep;

use super::{
    circuit::CircuitLayoutProvider,
    telemetry_provider::{TelemetryProvider, TelemetryUpdate},
    telemetry_state::TelemetryState,
};
use crate::circuit::CircuitKey;

pub struct TelemetryEngine<T: TelemetryProvider, C: CircuitLayoutProvider> {
    pub state: Arc<RwLock<TelemetryState>>,
    pub telemetry_provider: T,
    pub circuit_provider: Arc<C>,
    pub delay: Duration,
}

impl<T: TelemetryProvider, C: CircuitLayoutProvider + 'static> TelemetryEngine<T, C> {
    pub fn new(f1_client: T, circuit_layout_provider: C) -> Self {
        TelemetryEngine {
            state: Arc::new(RwLock::new(TelemetryState::default())),
            telemetry_provider: f1_client,
            circuit_provider: Arc::new(circuit_layout_provider),
            delay: Duration::default(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.telemetry_provider.connect().await
    }

    pub fn get_state(&self) -> Arc<RwLock<TelemetryState>> {
        Arc::clone(&self.state)
    }

    pub async fn run(&mut self) {
        struct StoredUpdate {
            timestamp: Instant,
            update: TelemetryUpdate,
        }

        let mut queue: VecDeque<StoredUpdate> = VecDeque::new();

        loop {
            tokio::select! {
                update_opt = self.telemetry_provider.next_updates() => {
                    if let Some(mut update) = update_opt {
                            self.check_and_fetch_circuit_layout(&mut update);
                            let stored = StoredUpdate {
                                timestamp: Instant::now(),
                                update: update.clone()
                            };
                            queue.push_back(stored);
                    }
                }

                _ = sleep(Duration::from_millis(100)) => {
                    while let Some(update) = queue.front() &&
                        update.timestamp >= Instant::now() - self.delay &&
                            let Some(u) = queue.pop_front() {
                            self.apply_updates(u.update);
                    }
                }
            }
        }
    }

    fn check_and_fetch_circuit_layout(&self, update: &mut TelemetryUpdate) {
        if let Some(circuit_update) = &update.circuit {
            if let Some(prev_circuit) = &self.state.read().unwrap().circuit {
                if circuit_update.key != prev_circuit.key {
                    self.spawn_layout_fetch(circuit_update.key, circuit_update.year);
                }
            } else {
                self.spawn_layout_fetch(circuit_update.key, circuit_update.year);
            }
        }
    }

    fn spawn_layout_fetch(&self, circuit_key: CircuitKey, year: u32) {
        info!("Fetching layout for key: {} and year {}", circuit_key, year);
        let provider = Arc::clone(&self.circuit_provider);
        let state_arc = Arc::clone(&self.state);

        tokio::spawn(async move {
            match provider.fetch(circuit_key, year).await {
                Ok(layout) => {
                    let mut state_lock = state_arc.write().unwrap();
                    let update = TelemetryUpdate {
                        circuit_layout: Some(layout),
                        ..Default::default()
                    };
                    if state_lock.apply(update) {
                        state_lock.update_version += 1;
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

    fn apply_updates(&self, update: TelemetryUpdate) {
        let mut state_lock = self.state.write().unwrap();

        if state_lock.apply(update) {
            state_lock.update_version += 1;
        }
    }
}
