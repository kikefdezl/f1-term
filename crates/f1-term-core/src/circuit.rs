use std::future::Future;

#[derive(Debug, Default, Clone)]
pub struct Circuit {
    pub key: u32,
    pub short_name: String,
    pub layout: Option<CircuitLayout>,
}

#[derive(Debug, Default, Clone)]
pub struct CircuitLayout {
    pub x: Vec<i32>,
    pub y: Vec<i32>,
}

pub trait CircuitLayoutProvider {
    fn fetch(
        &self,
        circuit_key: u32,
        year: u32,
    ) -> impl Future<Output = anyhow::Result<CircuitLayout>> + Send;
}
