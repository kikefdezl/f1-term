use log::info;
use reqwest::Client;

use super::models::{IndexResponse, SessionRootIndex};
use crate::Result;

const BASE_URL: &str = "https://livetiming.formula1.com/static";

#[derive(Clone)]
pub struct F1ApiClient {
    client: Client,
}

impl Default for F1ApiClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl F1ApiClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_index(&self, year: u32) -> Result<IndexResponse> {
        let url = format!("{}/{}/Index.json", BASE_URL, year);
        info!("Fetching Index.json: {}", url);
        let resp = self.client.get(&url).send().await?.text().await?;
        match serde_json::from_str::<IndexResponse>(&resp) {
            Ok(v) => Ok(v),
            Err(e) => {
                log::error!("Failed to parse index JSON for {}: {}", year, e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn get_session_index(&self, session_path: &str) -> Result<SessionRootIndex> {
        let clean_path = session_path.trim_matches('/');
        let url = format!("{}/{}/Index.json", BASE_URL, clean_path);
        info!("Fetching session index: {}", url);
        let resp = self.client.get(&url).send().await?.text().await?;
        match serde_json::from_str::<SessionRootIndex>(&resp) {
            Ok(v) => Ok(v),
            Err(e) => {
                log::error!(
                    "Failed to parse SessionRootIndex from {}: {}\nResp: {}",
                    url,
                    e,
                    resp
                );
                Err(Box::new(e))
            }
        }
    }

    pub async fn get_json(&self, session_path: &str, file: &str) -> Result<serde_json::Value> {
        let clean_path = session_path.trim_matches('/');
        let clean_file = file.trim_matches('/');
        let url = format!("{}/{}/{}", BASE_URL, clean_path, clean_file);
        info!("Fetching base json: {}", url);
        let resp = self.client.get(&url).send().await?.json().await?;
        Ok(resp)
    }

    pub async fn get_json_stream(&self, session_path: &str, file: &str) -> Result<String> {
        let clean_path = session_path.trim_matches('/');
        let clean_file = file.trim_matches('/');
        let url = format!("{}/{}/{}", BASE_URL, clean_path, clean_file);
        info!("Fetching jsonStream: {}", url);
        let resp = self.client.get(&url).send().await?.text().await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::client::F1ApiClient;

    #[tokio::test]
    async fn test_fetch_index() {
        let client = F1ApiClient::new();
        let response = client.get_index(2026).await.unwrap();
        assert_eq!(response.year, 2026);
    }
}
