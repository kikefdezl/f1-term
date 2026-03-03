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
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct TrackPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CircuitFeature {
    pub angle: f64,
    pub length: f64,
    pub number: i32,
    pub track_position: TrackPosition,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct CandidateLap {
    pub driver_number: String,
    pub lap_number: i32,
    pub lap_start_date: String,
    pub lap_start_session_time: f64,
    pub lap_time: f64,
    pub session: String,
    pub session_start_time: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MultiviewerCircuitResponse {
    pub candidate_lap: Option<CandidateLap>,
    pub circuit_key: u32,
    pub circuit_name: String,
    pub corners: Vec<CircuitFeature>,
    pub country_ioc_code: String,
    pub country_key: u32,
    pub country_name: String,
    pub location: String,
    pub marshal_lights: Vec<CircuitFeature>,
    pub marshal_sectors: Vec<CircuitFeature>,
    pub meeting_key: String,
    pub meeting_name: String,
    pub meeting_official_name: Option<String>,
    pub mini_sectors_indexes: Vec<usize>,
    pub race_date: String,
    pub rotation: f64,
    pub round: u32,
    pub x: Vec<i32>,
    pub y: Vec<i32>,
    pub year: u32,
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
                rotation: response.rotation,
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
