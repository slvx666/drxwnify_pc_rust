use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ArtistRef {
    pub name: String,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AlbumRef {
    pub title: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub year: Option<i32>,
    #[serde(default)]
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub artists: Vec<ArtistRef>,
    #[serde(default)]
    pub album: Option<AlbumRef>,
    #[serde(default)]
    pub duration_sec: i64,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub explicit: bool,
    #[serde(default)]
    pub youtube_id: Option<String>,
    #[serde(default)]
    pub isrc: Option<String>,
    #[serde(default)]
    pub local_path: Option<String>,
    #[serde(default)]
    pub bitrate_kbps: Option<u32>,
    #[serde(default)]
    pub source: Option<String>,
}

impl Track {
    pub fn new(id: impl Into<String>, title: impl Into<String>, artist: impl Into<String>, duration_sec: i64) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artists: vec![ArtistRef {
                name: artist.into(),
                id: None,
            }],
            album: None,
            duration_sec,
            cover: None,
            explicit: false,
            youtube_id: None,
            isrc: None,
            local_path: None,
            bitrate_kbps: None,
            source: None,
        }
    }

    pub fn artist_names(&self) -> String {
        if self.artists.is_empty() {
            "Unknown Artist".to_string()
        } else {
            self.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
        }
    }

    pub fn formatted_duration(&self) -> String {
        let mins = self.duration_sec / 60;
        let secs = self.duration_sec % 60;
        format!("{}:{:02}", mins, secs)
    }
}
