use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::{LyricLine, LyricsData, Track};
use regex::Regex;
use serde_json::Value;

pub struct LyricsService {
    http: HttpClient,
}

impl LyricsService {
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    pub async fn fetch_lyrics(&self, track: &Track) -> DrxwnifyResult<LyricsData> {
        // Try LRCLIB API first
        let url = format!(
            "https://lrclib.net/api/get?artist_name={}&track_name={}&duration={}",
            urlencoding::encode(&track.artist_names()),
            urlencoding::encode(&track.title),
            track.duration_sec
        );

        if let Ok(resp) = self.http.inner().get(&url).send().await {
            if resp.status().is_success() {
                if let Ok(val) = resp.json::<Value>().await {
                    if let Some(synced) = val["syncedLyrics"].as_str() {
                        let lines = Self::parse_lrc(synced);
                        return Ok(LyricsData {
                            track_id: track.id.clone(),
                            provider: "lrclib".to_string(),
                            is_synced: true,
                            lines,
                            plain_text: val["plainLyrics"].as_str().map(|s| s.to_string()),
                        });
                    }
                }
            }
        }

        Err(DrxwnifyError::Resolution {
            id: track.id.clone(),
            reason: "No lyrics found for track".to_string(),
        })
    }

    pub fn parse_lrc(lrc_text: &str) -> Vec<LyricLine> {
        let re = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)").unwrap();
        let mut lines = Vec::new();

        for line in lrc_text.lines() {
            if let Some(cap) = re.captures(line) {
                let mins: i64 = cap[1].parse().unwrap_or(0);
                let secs: i64 = cap[2].parse().unwrap_or(0);
                let ms_str = &cap[3];
                let ms: i64 = if ms_str.len() == 2 {
                    ms_str.parse::<i64>().unwrap_or(0) * 10
                } else {
                    ms_str.parse().unwrap_or(0)
                };

                let time_ms = (mins * 60 + secs) * 1000 + ms;
                let text = cap[4].trim().to_string();

                lines.push(LyricLine {
                    time_ms,
                    text,
                    translation: None,
                });
            }
        }

        lines
    }

    pub async fn translate_batch(&self, texts: &[String], target_lang: &str) -> DrxwnifyResult<Vec<String>> {
        let mut results = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(15) {
            let combined = chunk.join("\n");
            let url = format!(
                "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
                target_lang,
                urlencoding::encode(&combined)
            );

            if let Ok(val) = self.http.get_json::<Value>(&url).await {
                if let Some(arr) = val.get(0).and_then(|v| v.as_array()) {
                    let mut translated_full = String::new();
                    for item in arr {
                        if let Some(part) = item.get(0).and_then(|v| v.as_str()) {
                            translated_full.push_str(part);
                        }
                    }
                    for line in translated_full.lines() {
                        results.push(line.to_string());
                    }
                    continue;
                }
            }

            // Fallback: keep original if translation fails
            for t in chunk {
                results.push(t.clone());
            }
        }

        Ok(results)
    }
}
