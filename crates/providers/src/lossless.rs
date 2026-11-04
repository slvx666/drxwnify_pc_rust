use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::{AudioCodec, ResolvedStream, StreamProtocol, Track};
use serde_json::Value;

pub struct LosslessMirrorProvider {
    http: HttpClient,
    mirrors: Vec<String>,
}

impl LosslessMirrorProvider {
    pub fn new(http: HttpClient) -> Self {
        Self {
            http,
            mirrors: vec![
                "https://flac.soundbox.is".to_string(),
                "https://api.lossless.club".to_string(),
            ],
        }
    }

    pub async fn resolve_track(&self, track: &Track) -> DrxwnifyResult<ResolvedStream> {
        let isrc = match &track.isrc {
            Some(i) if !i.is_empty() => i.clone(),
            _ => {
                return Err(DrxwnifyError::Resolution {
                    id: track.id.clone(),
                    reason: "No ISRC code available for Lossless Hi-Fi mirror verification".to_string(),
                });
            }
        };

        for mirror in &self.mirrors {
            let url = format!("{}/track/by-isrc/{}", mirror, isrc);
            if let Ok(resp) = self.http.inner().get(&url).send().await {
                if resp.status().is_success() {
                    if let Ok(val) = resp.json::<Value>().await {
                        if let Some(stream_url) = val["streamUrl"].as_str() {
                            return Ok(ResolvedStream {
                                url: stream_url.to_string(),
                                source: "lossless".to_string(),
                                bitrate_kbps: 1411,
                                codec: AudioCodec::Flac,
                                protocol: StreamProtocol::DirectHttp,
                                headers: vec![],
                                expires_at: None,
                            });
                        }
                    }
                }
            }
        }

        Err(DrxwnifyError::Resolution {
            id: track.id.clone(),
            reason: "Lossless FLAC not found on active mirrors".to_string(),
        })
    }
}
