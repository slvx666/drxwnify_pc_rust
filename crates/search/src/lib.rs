use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub query: String,
    pub tracks: Vec<Track>,
    pub albums: Vec<AlbumRef>,
    pub artists: Vec<ArtistRef>,
    pub playlists: Vec<Playlist>,
}

pub struct SearchService {
    http: HttpClient,
}

impl SearchService {
    pub fn new(http: HttpClient) -> Self {
        Self { http }
    }

    pub async fn search(&self, query: &str) -> DrxwnifyResult<SearchResults> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(SearchResults::default());
        }

        // Mock/Spotify GQL Search port
        let mut results = SearchResults {
            query: q.to_string(),
            tracks: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
        };

        // Populate search tracks matching query
        for i in 1..=10 {
            results.tracks.push(Track::new(
                format!("search:{}:{}", q, i),
                format!("{} - Result #{}", q, i),
                format!("Artist {}", i),
                180 + (i as i64 * 12),
            ));
        }

        Ok(results)
    }
}
