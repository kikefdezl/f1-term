use std::future::Future;
use std::ops::Range;

use f1_term_core::circuit::{CircuitKey, CircuitLayout, CircuitLayoutProvider, Coord, Corner};
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

impl From<&TrackPosition> for Coord {
    fn from(value: &TrackPosition) -> Self {
        Coord {
            x: value.x,
            y: value.y,
        }
    }
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
    pub meeting_key: Option<String>,
    pub meeting_name: Option<String>,
    pub meeting_official_name: Option<String>,
    #[serde(default)]
    pub mini_sectors_indexes: Vec<usize>,
    pub race_date: String,
    pub rotation: f64,
    pub round: u32,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub year: u32,
}

impl CircuitLayoutProvider for MultiviewerClient {
    fn fetch(
        &self,
        circuit_key: CircuitKey,
        year: u32,
    ) -> impl Future<Output = Result<CircuitLayout, Box<dyn std::error::Error>>> + Send {
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

            let coords: Vec<Coord> = response
                .x
                .iter()
                .zip(response.y.iter())
                .map(|(x, y)| Coord {
                    x: *x as f64,
                    y: *y as f64,
                })
                .collect();
            let corners: Vec<Corner> = response
                .corners
                .iter()
                .map(|feat| Corner {
                    num: feat.number as u8,
                    coord: Coord::from(&feat.track_position),
                })
                .collect();

            let mut mini_sectors = Vec::with_capacity(response.mini_sectors_indexes.len());
            if !response.mini_sectors_indexes.is_empty() {
                mini_sectors.push(Range {
                    start: 0,
                    end: response.mini_sectors_indexes[0],
                });
                for i in 1..response.mini_sectors_indexes.len() {
                    mini_sectors.push(Range {
                        start: mini_sectors[i - 1].end + 1,
                        end: response.mini_sectors_indexes[i],
                    })
                }
            }

            Ok(CircuitLayout {
                coords,
                rotation: response.rotation,
                corners,
                mini_sectors,
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
        let layout = client
            .fetch(CircuitKey(7), 2024)
            .await
            .expect("Failed to fetch layout");

        assert_eq!(layout.coords.len(), 1005);

        assert_eq!(layout.mini_sectors.len(), 27);
        assert_eq!(layout.mini_sectors[0].start, 0);
        assert_eq!(layout.mini_sectors[0].end, 40);
        assert_eq!(layout.mini_sectors[26].start, 955);
        assert_eq!(layout.mini_sectors[26].end, 1004);
    }

    #[tokio::test]
    async fn test_fetch_circuit_layout_japan() {
        let client = MultiviewerClient::new();
        // Suzuka - missing some fields, coords are float, doesn't include miniSectorIndexes
        let layout = client
            .fetch(CircuitKey(46), 2026)
            .await
            .expect("Failed to fetch layout");
        assert_eq!(layout.coords.len(), 335);
    }
}
