use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::{AudioCodec, ResolvedStream, StreamProtocol, Track};
use regex::Regex;
use serde_json::Value;

pub const SOUNDCLOUD_TOLERANCE_SEC: i64 = 8;

pub struct SoundCloudProvider {
    http: HttpClient,
    client_id: Option<String>,
}

impl SoundCloudProvider {
    pub fn new(http: HttpClient) -> Self {
        Self { http, client_id: None }
    }

    pub async fn get_client_id(&mut self) -> DrxwnifyResult<String> {
        if let Some(ref id) = self.client_id {
            return Ok(id.clone());
        }

        let html = self.http.get_text("https://soundcloud.com").await?;
        let script_re = Regex::new(r#"src="(https://a-v2\.sndcdn\.com/assets/[^"]+\.js)""#).unwrap();
        let client_id_re = Regex::new(r#"client_id\s*[:=]\s*"?([0-9a-zA-Z]{32})"#).unwrap();

        let mut scripts = Vec::new();
        for cap in script_re.captures_iter(&html) {
            scripts.push(cap[1].to_string());
        }

        for script_url in scripts.into_iter().rev() {
            if let Ok(script_body) = self.http.get_text(&script_url).await {
                if let Some(id_cap) = client_id_re.captures(&script_body) {
                    let id = id_cap[1].to_string();
                    self.client_id = Some(id.clone());
                    return Ok(id);
                }
            }
        }

        Err(DrxwnifyError::Resolution {
            id: "soundcloud".to_string(),
            reason: "Failed to extract dynamic SoundCloud client_id".to_string(),
        })
    }

    pub async fn search_track(&mut self, track: &Track) -> DrxwnifyResult<ResolvedStream> {
        let client_id = self.get_client_id().await?;
        let query = format!("{} {}", track.artist_names(), track.title);
        let url = format!(
            "https://api-v2.soundcloud.com/search/tracks?q={}&client_id={}&limit=10",
            urlencoding::encode(&query),
            client_id
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

        if let Some(items) = resp["collection"].as_array() {
            for item in items {
                let duration_ms = item["duration"].as_i64().unwrap_or(0);
                let duration_sec = duration_ms / 1000;

                // 8 second tolerance filter
                if track.duration_sec > 0 && (duration_sec - track.duration_sec).abs() > SOUNDCLOUD_TOLERANCE_SEC {
                    continue;
                }

                if let Some(media) = item["media"]["transcodings"].as_array() {
                    // Prefer progressive direct MP3
                    for trans in media {
                        let protocol = trans["format"]["protocol"].as_str().unwrap_or("");
                        if protocol == "progressive" {
                            if let Some(trans_url) = trans["url"].as_str() {
                                let stream_req = format!("{}?client_id={}", trans_url, client_id);
                                let stream_resp: Result<Value, _> = self
                                    .http
                                    .inner()
                                    .get(&stream_req)
                                    .send()
                                    .await
                                    .map_err(|e| DrxwnifyError::Network(e.to_string()))?
                                    .json()
                                    .await;

                                if let Ok(resp) = stream_resp {
                                    if let Some(stream_url) = resp["url"].as_str() {
                                        return Ok(ResolvedStream {
                                            url: stream_url.to_string(),
                                            source: "soundcloud".to_string(),
                                            bitrate_kbps: 128,
                                            codec: AudioCodec::Mp3,
                                            protocol: StreamProtocol::DirectHttp,
                                            headers: vec![],
                                            expires_at: None,
                                        });
                                    }
                                }
                            }
                        }
                    }

                    // Fallback to HLS
                    for trans in media {
                        let protocol = trans["format"]["protocol"].as_str().unwrap_or("");
                        if protocol == "hls" {
                            if let Some(trans_url) = trans["url"].as_str() {
                                let stream_req = format!("{}?client_id={}", trans_url, client_id);
                                let stream_resp: Result<Value, _> = self
                                    .http
                                    .inner()
                                    .get(&stream_req)
                                    .send()
                                    .await
                                    .map_err(|e| DrxwnifyError::Network(e.to_string()))?
                                    .json()
                                    .await;

                                if let Ok(resp) = stream_resp {
                                    if let Some(stream_url) = resp["url"].as_str() {
                                        return Ok(ResolvedStream {
                                            url: stream_url.to_string(),
                                            source: "soundcloud".to_string(),
                                            bitrate_kbps: 128,
                                            codec: AudioCodec::Mp3,
                                            protocol: StreamProtocol::HlsAes128,
                                            headers: vec![],
                                            expires_at: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(DrxwnifyError::Resolution {
            id: track.id.clone(),
            reason: "No matching track found on SoundCloud within duration gate".to_string(),
        })
    }
}
