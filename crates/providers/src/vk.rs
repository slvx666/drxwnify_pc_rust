use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::{AudioCodec, ResolvedStream, StreamProtocol, Track};
use regex::Regex;
use serde_json::Value;

pub struct VkSourceProvider {
    http: HttpClient,
    token: Option<String>,
}

impl VkSourceProvider {
    pub fn new(http: HttpClient, token: Option<String>) -> Self {
        Self { http, token }
    }

    pub async fn search_track(&self, track: &Track) -> DrxwnifyResult<ResolvedStream> {
        let token = match &self.token {
            Some(t) if !t.is_empty() => t.as_str(),
            _ => return Err(DrxwnifyError::Resolution {
                id: track.id.clone(),
                reason: "No VK token available".to_string(),
            }),
        };

        let q = format!("{} {}", track.artist_names(), track.title);
        let url = format!(
            "https://api.vk.com/method/audio.search?v=5.116&access_token={}&q={}&count=15",
            token,
            urlencoding::encode(&q)
        );

        let resp: Value = self
            .http
            .inner()
            .get(&url)
            .header("User-Agent", "VKAndroidApp/5.52-4543 (Android 9; SDK 28; arm64-v8a; Xiaomi Redmi Note 7; ru)")
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        if let Some(items) = resp["response"]["items"].as_array() {
            let re_mp3 = Regex::new(r"/[0-9a-f]+(/audios)?/([0-9a-f]+)/index\.m3u8").unwrap();

            for item in items {
                let title = item["title"].as_str().unwrap_or("");
                let artist = item["artist"].as_str().unwrap_or("");
                let duration = item["duration"].as_i64().unwrap_or(0);
                let raw_url = item["url"].as_str().unwrap_or("");

                if raw_url.is_empty() {
                    continue;
                }

                // Check duration likeness (+- 15 seconds)
                if track.duration_sec > 0 && (duration - track.duration_sec).abs() > 15 {
                    continue;
                }

                // Try converting index.m3u8 to direct MP3
                let final_url = if raw_url.contains("index.m3u8") {
                    re_mp3.replace(raw_url, "/$2.mp3").to_string()
                } else {
                    raw_url.to_string()
                };

                let is_m3u8 = final_url.contains(".m3u8");

                return Ok(ResolvedStream {
                    url: final_url,
                    source: "vk".to_string(),
                    bitrate_kbps: 320,
                    codec: if is_m3u8 { AudioCodec::Aac } else { AudioCodec::Mp3 },
                    protocol: if is_m3u8 { StreamProtocol::HlsAes128 } else { StreamProtocol::DirectHttp },
                    headers: vec![(
                        "User-Agent".to_string(),
                        "VKAndroidApp/5.52-4543 (Android 9; SDK 28; arm64-v8a; Xiaomi Redmi Note 7; ru)".to_string(),
                    )],
                    expires_at: None,
                });
            }
        }

        Err(DrxwnifyError::Resolution {
            id: track.id.clone(),
            reason: "No matching track found in VK Music".to_string(),
        })
    }
}
