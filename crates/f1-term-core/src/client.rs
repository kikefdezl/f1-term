use crate::snapshot::FullSnapshot;

#[derive(Debug)]
pub enum TelemetryEvent {
    Full(FullSnapshot),
    Empty,
}

pub trait F1Client {
    fn connect(&mut self) -> impl Future<Output = Result<(), Box<dyn std::error::Error>>>;
    fn next_event(&mut self) -> impl Future<Output = Option<TelemetryEvent>>;
}
