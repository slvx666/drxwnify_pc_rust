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
        let tracks = self.search_tracks(query).await?;
        if let Some(first) = tracks.first() {
            if let Some(ref vid) = first.youtube_id {
                return Ok(vid.clone());
            }
        }

        Err(DrxwnifyError::Resolution {
            id: query.to_string(),
            reason: "No matching YouTube Music track found".to_string(),
        })
    }

    pub async fn search_tracks(&self, query: &str) -> DrxwnifyResult<Vec<Track>> {
        let search_url = "https://music.youtube.com/youtubei/v1/search?alt=json";
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
            .post(search_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        let mut tracks = Vec::new();
        extract_ytm_tracks(&resp, &mut tracks);
        if tracks.is_empty() {
            // Fallback search video id
            if let Some(vid) = find_first_video_id(&resp) {
                let mut t = Track::new(format!("yt:{}", vid), query, "YouTube Music", 180);
                t.youtube_id = Some(vid);
                tracks.push(t);
            }
        }

        Ok(tracks)
    }

    pub async fn get_stream_for_video(&self, video_id: &str) -> DrxwnifyResult<ResolvedStream> {
        let player_url = "https://music.youtube.com/youtubei/v1/player";

        // Try ANDROID_VR 1.65.10 with visitorData
        let mut vr_client = serde_json::json!({
            "clientName": "ANDROID_VR",
            "clientVersion": "1.65.10",
            "deviceMake": "Oculus",
            "deviceModel": "Quest 3",
            "osName": "Android",
            "osVersion": "12L",
            "hl": "en",
            "gl": "US"
        });

        // If visitorData is present from search or previous call, set it
        let visitor_id = "CgtLWlY1UV9PNjdnQSjy6LLXBjIKCgJERRIEEgAga0gBYqoCCqcCQ3RvQkNBVVNqQUVCREI2UERNdGFzcW1sa195ZVNqajYwYmRGT2p6MEVNS1drTVJrQU5NTXlJVEVNMFRvbkZiSW5xTUpEbVNRcDZjNnN";
        vr_client["visitorData"] = serde_json::Value::String(visitor_id.to_string());

        let vr_body = serde_json::json!({
            "context": {
                "client": vr_client
            },
            "videoId": video_id
        });

        let raw_resp = self
            .http
            .inner()
            .post(player_url)
            .header("User-Agent", "com.google.android.apps.youtube.vr.oculus/1.65.10 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip")
            .header("X-Goog-Api-Format-Version", "1")
            .header("X-YouTube-Client-Name", "28")
            .header("X-YouTube-Client-Version", "1.65.10")
            .header("X-Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/")
            .header("X-Goog-Visitor-Id", visitor_id)
            .json(&vr_body)
            .send()
            .await;

        match raw_resp {
            Ok(r) => {
                let status = r.status();
                let txt = r.text().await.unwrap_or_default();
                println!("DEBUG ANDROID_VR status: {}, body len: {}, first 200 chars: {}", status, txt.len(), &txt[..txt.len().min(200)]);
                if let Ok(resp) = serde_json::from_str::<Value>(&txt) {
                    println!("DEBUG ANDROID_VR playability: {:?}", resp["playabilityStatus"]);
                    if let Some(formats) = resp["streamingData"]["adaptiveFormats"].as_array() {
                        println!("DEBUG ANDROID_VR formats count: {}", formats.len());
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
                    }
                }
            }
            Err(e) => {
                println!("DEBUG ANDROID_VR send err: {e}");
            }
        }

        // Fallback: VisionOS Client
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
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15")
            .header("X-Goog-Api-Format-Version", "1")
            .header("X-YouTube-Client-Name", "101")
            .header("X-YouTube-Client-Version", "0.1")
            .header("X-Origin", "https://music.youtube.com")
            .header("Referer", "https://music.youtube.com/")
            .json(&body)
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        println!("DEBUG YouTube response playability: {:?}", resp["playabilityStatus"]);
        println!("DEBUG YouTube response has streamingData: {}", resp["streamingData"].is_object());
        if let Some(formats) = resp["streamingData"]["adaptiveFormats"].as_array() {
            println!("DEBUG formats count: {}", formats.len());
            for f in formats {
                println!("DEBUG itag: {:?}, mime: {:?}, has_url: {}, has_sig: {}", f["itag"], f["mimeType"], f["url"].is_string(), f["signatureCipher"].is_string());
            }

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

fn extract_ytm_tracks(v: &Value, out: &mut Vec<Track>) {
    if let Some(obj) = v.as_object() {
        if let Some(renderer) = obj.get("musicResponsiveListItemRenderer") {
            let video_id = renderer
                .pointer("/playlistItemData/videoId")
                .or_else(|| renderer.pointer("/navigationEndpoint/watchEndpoint/videoId"))
                .and_then(|x| x.as_str());

            if let Some(vid) = video_id {
                let title = renderer
                    .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
                    .and_then(|x| x.as_str())
                    .unwrap_or("Unknown Title");

                let artist = renderer
                    .pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
                    .and_then(|x| x.as_str())
                    .unwrap_or("Unknown Artist");

                let cover = renderer
                    .pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails")
                    .and_then(|arr| arr.as_array())
                    .and_then(|arr| arr.last())
                    .and_then(|thumb| thumb.get("url"))
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());

                let mut track = Track::new(format!("yt:{}", vid), title, artist, 180);
                track.youtube_id = Some(vid.to_string());
                track.cover = cover;
                out.push(track);
            }
        }

        for (_, val) in obj {
            extract_ytm_tracks(val, out);
        }
    } else if let Some(arr) = v.as_array() {
        for val in arr {
            extract_ytm_tracks(val, out);
        }
    }
}
