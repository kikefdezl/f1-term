use std::{future::Future, sync::Arc};

use crate::session::Session;

#[derive(Debug)]
pub enum TelemetryEvent {
    SessionUpdate(Arc<Session>),
    Empty,
}

pub trait F1Client {
    fn connect(&mut self) -> impl Future<Output = Result<(), Box<dyn std::error::Error>>>;
    fn next_event(&mut self) -> impl Future<Output = Option<TelemetryEvent>>;
}
