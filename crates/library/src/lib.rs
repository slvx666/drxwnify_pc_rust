use drxwnify_core::{AppPaths, DrxwnifyResult};
use drxwnify_persistence::Store;
use drxwnify_shared_models::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct LibraryManager {
    paths: AppPaths,
    data: Arc<RwLock<LibraryData>>,
}

impl LibraryManager {
    pub fn new(paths: AppPaths) -> Self {
        let data: LibraryData = Store::load_or_default(&paths.library_file);
        Self {
            paths,
            data: Arc::new(RwLock::new(data)),
        }
    }

    pub fn is_liked(&self, track_id: &str) -> bool {
        self.data.read().liked.iter().any(|t| t.id == track_id)
    }

    pub fn toggle_like(&self, track: Track) -> DrxwnifyResult<bool> {
        let is_now_liked = {
            let mut data = self.data.write();
            if let Some(idx) = data.liked.iter().position(|t| t.id == track.id) {
                data.liked.remove(idx);
                false
            } else {
                data.liked.insert(0, track);
                true
            }
        };

        self.save()?;
        Ok(is_now_liked)
    }

    pub fn record_history(&self, track: Track) -> DrxwnifyResult<()> {
        {
            let mut data = self.data.write();
            data.history.insert(0, Played {
                track,
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
            if data.history.len() > 2000 {
                data.history.truncate(2000);
            }
        }
        self.save()?;
        Ok(())
    }

    pub fn get_liked(&self) -> Vec<Track> {
        self.data.read().liked.clone()
    }

    pub fn get_playlists(&self) -> Vec<Playlist> {
        self.data.read().playlists.clone()
    }

    pub fn get_history(&self) -> Vec<Played> {
        self.data.read().history.clone()
    }

    pub fn create_playlist(&self, name: &str) -> DrxwnifyResult<Playlist> {
        let pl = Playlist::new(format!("pl_{}", chrono::Utc::now().timestamp_millis()), name);
        {
            let mut data = self.data.write();
            data.playlists.push(pl.clone());
        }
        self.save()?;
        Ok(pl)
    }

    pub fn save(&self) -> DrxwnifyResult<()> {
        let data = self.data.read().clone();
        Store::save_atomic(&self.paths.library_file, &data)
    }
}
