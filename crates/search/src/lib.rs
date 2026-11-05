use drxwnify_core::DrxwnifyResult;
use drxwnify_network::HttpClient;
use drxwnify_providers::InnerTubeProvider;
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
    innertube: InnerTubeProvider,
}

impl SearchService {
    pub fn new(http: HttpClient) -> Self {
        let innertube = InnerTubeProvider::new(http.clone());
        Self { http, innertube }
    }

    pub async fn search(&self, query: &str) -> DrxwnifyResult<SearchResults> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(SearchResults::default());
        }

        let mut results = SearchResults {
            query: q.to_string(),
            tracks: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
        };

        // Search real YouTube Music InnerTube tracks
        match self.innertube.search_tracks(q).await {
            Ok(tracks) if !tracks.is_empty() => {
                results.tracks = tracks;
            }
            Err(e) => {
                tracing::warn!("InnerTube search error: {e}");
            }
            _ => {}
        }

        // Add matching artist/album references derived from tracks
        for t in &results.tracks {
            for a in &t.artists {
                if !results.artists.iter().any(|existing| existing.name == a.name) {
                    results.artists.push(a.clone());
                }
            }
            if let Some(ref alb) = t.album {
                if !results.albums.iter().any(|existing| existing.title == alb.title) {
                    results.albums.push(alb.clone());
                }
            }
        }

        Ok(results)
    }
}
