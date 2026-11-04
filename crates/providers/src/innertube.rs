use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::{AudioCodec, ResolvedStream, StreamProtocol, Track};
use serde_json::Value;

pub struct InnerTubeProvider {
    http: HttpClient,
}

impl InnerTubeProvider {
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    pub async fn resolve_track(&self, track: &Track) -> DrxwnifyResult<ResolvedStream> {
        let query = format!("{} {}", track.title, track.artist_names());
        let video_id = match &track.youtube_id {
            Some(id) if !id.is_empty() => id.clone(),
            _ => self.search_video_id(&query).await?,
        };

        self.get_stream_for_video(&video_id).await
    }

    pub async fn search_video_id(&self, query: &str) -> DrxwnifyResult<String> {
        let search_url = format!(
            "https://music.youtube.com/youtubei/v1/search?alt=json"
        );
        let body = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20240901.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "query": query
        });

        let resp: Value = self
            .http
            .inner()
            .post(&search_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        // Find first track videoId in contents
        if let Some(video_id) = find_first_video_id(&resp) {
            return Ok(video_id);
        }

        Err(DrxwnifyError::Resolution {
            id: query.to_string(),
            reason: "No matching YouTube Music track found".to_string(),
        })
    }

    pub async fn get_stream_for_video(&self, video_id: &str) -> DrxwnifyResult<ResolvedStream> {
        // VisionOS Client emulation
        let player_url = "https://www.youtube.com/youtubei/v1/player?alt=json";
        let body = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "VISIONOS",
                    "clientVersion": "0.1",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id
        });

        let resp: Value = self
            .http
            .inner()
            .post(player_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        if let Some(formats) = resp["streamingData"]["adaptiveFormats"].as_array() {
            // Find itag 251 (Opus ~160k) or 140 (AAC ~128k)
            for fmt in formats {
                let itag = fmt["itag"].as_u64().unwrap_or(0);
                if itag == 251 {
                    if let Some(url) = fmt["url"].as_str() {
                        return Ok(ResolvedStream {
                            url: url.to_string(),
                            source: "youtube".to_string(),
                            bitrate_kbps: 160,
                            codec: AudioCodec::Opus,
                            protocol: StreamProtocol::DirectHttp,
                            headers: vec![],
                            expires_at: None,
                        });
                    }
                }
            }

            for fmt in formats {
                let itag = fmt["itag"].as_u64().unwrap_or(0);
                if itag == 140 {
                    if let Some(url) = fmt["url"].as_str() {
                        return Ok(ResolvedStream {
                            url: url.to_string(),
                            source: "youtube".to_string(),
                            bitrate_kbps: 128,
                            codec: AudioCodec::Aac,
                            protocol: StreamProtocol::DirectHttp,
                            headers: vec![],
                            expires_at: None,
                        });
                    }
                }
            }
        }

        Err(DrxwnifyError::Resolution {
            id: video_id.to_string(),
            reason: "No suitable audio format (itag 251/140) found in YouTube response".to_string(),
        })
    }
}

fn find_first_video_id(v: &Value) -> Option<String> {
    if let Some(obj) = v.as_object() {
        if let Some(id) = obj.get("videoId").and_then(|x| x.as_str()) {
            return Some(id.to_string());
        }
        for (_, val) in obj {
            if let Some(id) = find_first_video_id(val) {
                return Some(id);
            }
        }
    } else if let Some(arr) = v.as_array() {
        for val in arr {
            if let Some(id) = find_first_video_id(val) {
                return Some(id);
            }
        }
    }
    None
}
