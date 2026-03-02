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
            for update in updates {
                match update {
                    TelemetryUpdate::SessionInfo(info) => {
                        let circuit_key = info.meeting.circuit.key;
                        let year = info.start_date.year() as u32;

                        {
                            let mut state_lock = self.state.write().unwrap();
                            state_lock.info = Some(*info);
                        }

                        if circuit_key != self._cached_circuit_key
                            && let Ok(layout) = self.circuit_provider.fetch(circuit_key, year).await
                        {
                            let mut state_lock = self.state.write().unwrap();
                            if let Some(info_mut) = &mut state_lock.info {
                                info_mut.meeting.circuit.layout = Some(layout);
                            }
                            self._cached_circuit_key = circuit_key;
                        }
                    }
                    TelemetryUpdate::DriverList(drivers, teams) => {
                        let mut state_lock = self.state.write().unwrap();
                        state_lock.drivers = drivers;
                        state_lock.teams = teams;
                    }
                    TelemetryUpdate::TimingData(timing_data) => {
                        let mut state_lock = self.state.write().unwrap();
                        state_lock.timing_data = timing_data;
                    }
                    TelemetryUpdate::Stints(stints) => {
                        let mut state_lock = self.state.write().unwrap();
                        state_lock.stints = stints;
                    }
                    TelemetryUpdate::TrackStatus(track_status) => {
                        let mut state_lock = self.state.write().unwrap();
                        state_lock.track_status = Some(track_status);
                    }
                    TelemetryUpdate::RaceControlMessages(messages) => {
                        let mut state_lock = self.state.write().unwrap();
                        state_lock.race_control_messages = messages;
                    }
                    TelemetryUpdate::Weather(weather) => {
                        let mut state_lock = self.state.write().unwrap();
                        state_lock.weather = Some(weather);
                    }
                    TelemetryUpdate::Empty => {}
                }
            }
        }
    }
}
