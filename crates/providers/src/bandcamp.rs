use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::{AudioCodec, ResolvedStream, StreamProtocol, Track};
use regex::Regex;
use serde_json::Value;

pub struct BandcampProvider {
    http: HttpClient,
}

impl BandcampProvider {
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    pub async fn search_track(&self, track: &Track) -> DrxwnifyResult<ResolvedStream> {
        let q = format!("{} {}", track.artist_names(), track.title);
        let url = format!(
            "https://bandcamp.com/api/fuzzysearch/1/autocomplete?q={}",
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

        if let Some(results) = resp["auto"]["results"].as_array() {
            for res in results {
                let r_type = res["type"].as_str().unwrap_or("");
                if r_type == "t" {
                    if let Some(track_url) = res["url"].as_str() {
                        let html = self.http.get_text(track_url).await?;
                        let re_stream = Regex::new(r#""mp3-128":"([^"]+)""#).unwrap();
                        if let Some(cap) = re_stream.captures(&html) {
                            return Ok(ResolvedStream {
                                url: cap[1].replace(r"\u0026", "&"),
                                source: "bandcamp".to_string(),
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

        Err(DrxwnifyError::Resolution {
            id: track.id.clone(),
            reason: "Track not found on Bandcamp".to_string(),
        })
    }
}
