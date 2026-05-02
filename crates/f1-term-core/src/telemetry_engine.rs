use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use log::info;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{Instant, interval};

use super::circuit::CircuitLayoutProvider;
use super::telemetry_provider::{TelemetryProvider, TelemetryUpdate};
use super::telemetry_state::TelemetryState;
use crate::aggregators::TelemetryAggregator;
use crate::aggregators::layout_fetch::LayoutFetchAggregator;
use crate::aggregators::sector_count::SectorCountAggregator;
use crate::circuit::CircuitKey;

pub enum EngineTask {
    FetchCircuitLayout,
}

pub enum EngineCommand {
    IncreaseDelay(Duration),
    DecreaseDelay(Duration),
}

pub struct TelemetryEngine<T: TelemetryProvider, C: CircuitLayoutProvider> {
    pub state: Arc<RwLock<TelemetryState>>,
    pub telemetry_provider: T,
    pub circuit_provider: Arc<C>,
    aggregators: Vec<Box<dyn TelemetryAggregator>>,
}

impl<T: TelemetryProvider, C: CircuitLayoutProvider + 'static> TelemetryEngine<T, C> {
    pub fn new(f1_client: T, circuit_layout_provider: C) -> Self {
        TelemetryEngine {
            state: Arc::new(RwLock::new(TelemetryState::default())),
            telemetry_provider: f1_client,
            circuit_provider: Arc::new(circuit_layout_provider),
            aggregators: vec![
                Box::new(LayoutFetchAggregator),
                Box::new(SectorCountAggregator),
            ],
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.telemetry_provider.connect().await
    }

    pub fn get_state(&self) -> Arc<RwLock<TelemetryState>> {
        Arc::clone(&self.state)
    }

    pub async fn run(&mut self, mut cmd_rx: UnboundedReceiver<EngineCommand>) {
        struct StoredUpdate {
            timestamp: Instant,
            update: TelemetryUpdate,
        }

        let mut queue: VecDeque<StoredUpdate> = VecDeque::new();
        let mut interval = interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                cmd_opt = cmd_rx.recv() => {
                    match cmd_opt {
                        Some(cmd) => match cmd {
                            EngineCommand::IncreaseDelay(a) => self.increase_delay(a),
                            EngineCommand::DecreaseDelay(a) => self.decrease_delay(a),
                        },
                        None => break,
                    }
                }


                update_opt = self.telemetry_provider.next_updates() => {
                    if let Some(update) = update_opt {
                        let stored = StoredUpdate {
                            timestamp: Instant::now(),
                            update,
                        };
                        queue.push_back(stored);
                    }
                }

                _ = interval.tick() => {
                    while let Some(update) = queue.front() {
                        if update.timestamp.elapsed() >= self.state.read().unwrap().delay {
                            if let Some(ready_update) = queue.pop_front() {
                                self.process_pipeline(ready_update.update);
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn process_pipeline(&self, mut update: TelemetryUpdate) {
        let mut tasks = vec![];
        let state = self.state.read().unwrap();
        for aggregator in &self.aggregators {
            tasks.extend(aggregator.process(&state, &mut update));
        }
        drop(state);

        // TODO: It's not right that the execute_tasks() knows about fields that are required by
        // some of the aggregators (circuit_info), it should be generic.
        // This should be refactored.
        let circuit_info = update.circuit.as_ref().map(|c| (c.key, c.year));
        self.apply_updates(update);
        self.execute_tasks(tasks, circuit_info);
    }

    fn execute_tasks(&self, tasks: Vec<EngineTask>, circuit_info: Option<(CircuitKey, u32)>) {
        for task in tasks {
            match task {
                EngineTask::FetchCircuitLayout => {
                    if let Some((key, year)) = circuit_info {
                        self.spawn_layout_fetch(key, year)
                    }
                }
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

    fn increase_delay(&mut self, amount: Duration) {
        let mut state = self.state.write().unwrap();
        state.delay += amount;
        state.update_version += 1;
    }

    fn decrease_delay(&mut self, amount: Duration) {
        let mut state = self.state.write().unwrap();
        state.delay = state.delay.saturating_sub(amount);
        state.update_version += 1;
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;
    use tokio::time::advance;

    use super::*;
    use crate::circuit::{Circuit, MockCircuitLayoutProvider};
    use crate::telemetry_provider::{MockTelemetryProvider, TelemetryUpdate};

    struct TestFixture {
        state_ref: Arc<RwLock<TelemetryState>>,
        update_tx: mpsc::UnboundedSender<TelemetryUpdate>,
        cmd_tx: mpsc::UnboundedSender<EngineCommand>,
        engine_task: tokio::task::JoinHandle<()>,
    }

    impl TestFixture {
        fn setup() -> Self {
            let (update_tx, rx) = mpsc::unbounded_channel();
            let telemetry_provider = MockTelemetryProvider { rx };
            let circuit_provider = MockCircuitLayoutProvider;

            let mut engine = TelemetryEngine::new(telemetry_provider, circuit_provider);
            let state_ref = engine.get_state();
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

            let engine_task = tokio::spawn(async move {
                engine.run(cmd_rx).await;
            });

            Self {
                state_ref,
                update_tx,
                cmd_tx,
                engine_task,
            }
        }

        fn with_delay(self, delay: Duration) -> Self {
            {
                let mut state = self.state_ref.write().unwrap();
                state.delay = delay;
            }
            self
        }

        fn with_circuit(self, circuit: Circuit) -> Self {
            {
                let mut state = self.state_ref.write().unwrap();
                state.circuit = Some(circuit);
            }
            self
        }

        async fn teardown(self) {
            drop(self.cmd_tx);
            self.engine_task.await.unwrap();
        }
    }

    #[tokio::test(start_paused = true)]
    async fn test_telemetry_delay_mechanism_before_delay() {
        let fixture = TestFixture::setup().with_delay(Duration::from_secs(5));

        let update = TelemetryUpdate {
            track_status: Some(crate::track_status::TrackStatus {
                status: 1,
                message: "All Clear".to_string(),
            }),
            ..Default::default()
        };
        fixture.update_tx.send(update).unwrap();

        tokio::task::yield_now().await;

        {
            let state = fixture.state_ref.read().unwrap();
            assert_eq!(state.update_version, 0);
        }
        fixture.teardown().await;
    }

    #[tokio::test(start_paused = true)]
    async fn test_telemetry_delay_mechanism_after_delay() {
        let fixture = TestFixture::setup().with_delay(Duration::from_secs(5));

        let update = TelemetryUpdate {
            track_status: Some(crate::track_status::TrackStatus {
                status: 1,
                message: "All Clear".to_string(),
            }),
            ..Default::default()
        };
        fixture.update_tx.send(update).unwrap();

        tokio::task::yield_now().await;
        advance(Duration::from_secs(5) + Duration::from_millis(200)).await;
        tokio::task::yield_now().await;

        {
            let state = fixture.state_ref.read().unwrap();
            assert!(state.update_version > 0);
        }

        fixture.teardown().await;
    }

    #[tokio::test(start_paused = true)]
    async fn test_increase_delay_command() {
        let fixture = TestFixture::setup();

        fixture
            .cmd_tx
            .send(EngineCommand::IncreaseDelay(Duration::from_secs(2)))
            .unwrap();

        tokio::task::yield_now().await;

        {
            let state = fixture.state_ref.read().unwrap();
            assert_eq!(state.delay, Duration::from_secs(2));
        }

        fixture.teardown().await;
    }

    #[tokio::test(start_paused = true)]
    async fn test_decrease_delay_command() {
        let fixture = TestFixture::setup().with_delay(Duration::from_secs(5));

        fixture
            .cmd_tx
            .send(EngineCommand::DecreaseDelay(Duration::from_secs(2)))
            .unwrap();

        tokio::task::yield_now().await;

        {
            let state = fixture.state_ref.read().unwrap();
            assert_eq!(state.delay, Duration::from_secs(3));
        }

        fixture.teardown().await;
    }

    #[tokio::test(start_paused = true)]
    async fn test_decrease_delay_saturates_at_zero() {
        let fixture = TestFixture::setup().with_delay(Duration::from_secs(2));

        fixture
            .cmd_tx
            .send(EngineCommand::DecreaseDelay(Duration::from_secs(5)))
            .unwrap();

        tokio::task::yield_now().await;

        {
            let state = fixture.state_ref.read().unwrap();
            assert_eq!(state.delay, Duration::from_secs(0));
        }

        fixture.teardown().await;
    }

    #[tokio::test(start_paused = true)]
    async fn test_fetches_circuit_layout_when_none() {
        let fixture = TestFixture::setup();

        {
            let state = fixture.state_ref.read().unwrap();
            assert!(state.circuit.is_none());
        }

        let update = TelemetryUpdate {
            circuit: Some(Circuit {
                key: CircuitKey(1),
                year: 2024,
                ..Default::default()
            }),
            ..Default::default()
        };
        fixture.update_tx.send(update).unwrap();

        tokio::task::yield_now().await;
        advance(Duration::from_millis(150)).await;
        tokio::task::yield_now().await;

        {
            let state = fixture.state_ref.read().unwrap();
            assert!(state.circuit.is_some());
            assert!(state.circuit.as_ref().unwrap().layout.is_some());
        }

        fixture.teardown().await;
    }

    #[tokio::test(start_paused = true)]
    async fn test_fetches_circuit_layout_when_different_key() {
        let fixture = TestFixture::setup().with_circuit(Circuit {
            key: CircuitKey(1),
            year: 2024,
            layout: None,
            ..Default::default()
        });

        let update = TelemetryUpdate {
            circuit: Some(Circuit {
                key: CircuitKey(2),
                year: 2024,
                ..Default::default()
            }),
            ..Default::default()
        };
        fixture.update_tx.send(update).unwrap();

        tokio::task::yield_now().await;
        advance(Duration::from_millis(150)).await;
        tokio::task::yield_now().await;

        {
            let state = fixture.state_ref.read().unwrap();
            assert_eq!(state.circuit.as_ref().unwrap().key, CircuitKey(2));
            assert!(state.circuit.as_ref().unwrap().layout.is_some());
        }

        fixture.teardown().await;
    }
}
