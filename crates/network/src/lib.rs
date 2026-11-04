use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> DrxwnifyResult<Self> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"),
        );

        let client = Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(8))
            .brotli(true)
            .gzip(true)
            .build()
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        Ok(Self { client })
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }

    pub async fn get_text(&self, url: &str) -> DrxwnifyResult<String> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(format!("GET {url} failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(DrxwnifyError::Network(format!("GET {url} status: {}", resp.status())));
        }

        resp.text()
            .await
            .map_err(|e| DrxwnifyError::Network(format!("Reading text from {url} failed: {e}")))
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> DrxwnifyResult<T> {
        let text = self.get_text(url).await?;
        serde_json::from_str(&text).map_err(DrxwnifyError::Json)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HttpClient")
    }
}
