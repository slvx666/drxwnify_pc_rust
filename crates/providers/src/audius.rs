use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::{AudioCodec, ResolvedStream, StreamProtocol, Track};
use serde_json::Value;

pub struct AudiusProvider {
    http: HttpClient,
    discovery_node: Option<String>,
}

impl AudiusProvider {
    pub fn new(http: HttpClient) -> Self {
        Self { http, discovery_node: None }
    }

    pub async fn get_node(&mut self) -> DrxwnifyResult<String> {
        if let Some(ref node) = self.discovery_node {
            return Ok(node.clone());
        }

        let resp: Value = self
            .http
            .inner()
            .get("https://api.audius.co")
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        if let Some(nodes) = resp["data"].as_array() {
            if let Some(first) = nodes.first().and_then(|n| n.as_str()) {
                let node = first.to_string();
                self.discovery_node = Some(node.clone());
                return Ok(node);
            }
        }

        let fallback = "https://discoveryprovider.audius.co".to_string();
        self.discovery_node = Some(fallback.clone());
        Ok(fallback)
    }

    pub async fn search_track(&mut self, track: &Track) -> DrxwnifyResult<ResolvedStream> {
        let node = self.get_node().await?;
        let q = format!("{} {}", track.artist_names(), track.title);
        let url = format!(
            "{}/v1/tracks/search?query={}&app_name=drxwnify",
            node,
            urlencoding::encode(&q)
        );

        let resp: Value = self
            .http
            .inner()
            .get(&url)
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        if let Some(data) = resp["data"].as_array() {
            if let Some(first_track) = data.first() {
                if let Some(track_id) = first_track["id"].as_str() {
                    let stream_url = format!("{}/v1/tracks/{}/stream?app_name=drxwnify", node, track_id);
                    return Ok(ResolvedStream {
                        url: stream_url,
                        source: "audius".to_string(),
                        bitrate_kbps: 128,
                        codec: AudioCodec::Mp3,
                        protocol: StreamProtocol::DirectHttp,
                        headers: vec![],
                        expires_at: None,
                    });
                }
            }
        }

        Err(DrxwnifyError::Resolution {
            id: track.id.clone(),
            reason: "Track not found on Audius".to_string(),
        })
    }
}
