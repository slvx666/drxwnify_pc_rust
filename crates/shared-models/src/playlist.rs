use crate::track::Track;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub cover: Option<String>,
    #[serde(default)]
    pub created: i64,
    #[serde(default)]
    pub pinned: bool,
}

impl Playlist {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            tracks: Vec::new(),
            cover: None,
            created: chrono::Utc::now().timestamp_millis(),
            pinned: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Played {
    pub track: Track,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryData {
    #[serde(default)]
    pub liked: Vec<Track>,
    #[serde(default)]
    pub playlists: Vec<Playlist>,
    #[serde(default)]
    pub saved_albums: Vec<crate::track::AlbumRef>,
    #[serde(default)]
    pub followed_artists: Vec<crate::track::ArtistRef>,
    #[serde(default)]
    pub history: Vec<Played>,
}
