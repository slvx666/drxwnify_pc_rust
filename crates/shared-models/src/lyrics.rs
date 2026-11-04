use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time_ms: i64,
    pub text: String,
    #[serde(default)]
    pub translation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LyricsData {
    pub track_id: String,
    pub provider: String,
    pub is_synced: bool,
    #[serde(default)]
    pub lines: Vec<LyricLine>,
    #[serde(default)]
    pub plain_text: Option<String>,
}
