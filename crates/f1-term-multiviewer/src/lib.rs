use std::future::Future;

use f1_term_core::circuit::{CircuitLayout, CircuitLayoutProvider};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct MultiviewerClient {
    client: Client,
}

impl MultiviewerClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for MultiviewerClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct MultiviewerCircuitResponse {
    x: Vec<i32>,
    y: Vec<i32>,
}

impl CircuitLayoutProvider for MultiviewerClient {
    fn fetch(
        &self,
        circuit_key: u32,
        year: u32,
    ) -> impl Future<Output = anyhow::Result<CircuitLayout>> + Send {
        let client = self.client.clone();
        async move {
            let url = format!(
                "https://api.multiviewer.app/api/v1/circuits/{}/{}",
                circuit_key, year
            );

            let response = client
                .get(&url)
                .send()
                .await?
                .error_for_status()?
                .json::<MultiviewerCircuitResponse>()
                .await?;

            Ok(CircuitLayout {
                x: response.x,
                y: response.y,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_circuit_layout() {
        let client = MultiviewerClient::new();
        // Spa-Francorchamps (circuit key 7) for 2024
        let layout = client.fetch(7, 2024).await.expect("Failed to fetch layout");

        assert!(!layout.x.is_empty(), "x coordinates should not be empty");
        assert!(!layout.y.is_empty(), "y coordinates should not be empty");

        assert_eq!(
            layout.x.len(),
            layout.y.len(),
            "x and y coordinate arrays should be the same length"
        );
    }
}
