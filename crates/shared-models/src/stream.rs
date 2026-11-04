use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AudioCodec {
    Opus,
    Aac,
    Mp3,
    Flac,
    Vorbis,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StreamProtocol {
    DirectHttp,
    HlsAes128,
    LocalFile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedStream {
    pub url: String,
    pub source: String,
    pub bitrate_kbps: u32,
    pub codec: AudioCodec,
    pub protocol: StreamProtocol,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub expires_at: Option<i64>,
}
