use drxwnify_core::{AppPaths, DrxwnifyResult};
use std::path::PathBuf;

pub struct AudioCache {
    paths: AppPaths,
}

impl AudioCache {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    pub fn get_cached_path(&self, track_id: &str) -> Option<PathBuf> {
        let clean_id = track_id.replace([':', '/', '\\'], "_");
        let path = self.paths.audio_cache_dir.join(format!("{}.audio", clean_id));
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn get_part_path(&self, track_id: &str) -> PathBuf {
        let clean_id = track_id.replace([':', '/', '\\'], "_");
        self.paths.audio_cache_dir.join(format!("{}.part", clean_id))
    }

    pub fn commit_cache(&self, track_id: &str) -> DrxwnifyResult<PathBuf> {
        let part = self.get_part_path(track_id);
        let clean_id = track_id.replace([':', '/', '\\'], "_");
        let dest = self.paths.audio_cache_dir.join(format!("{}.audio", clean_id));
        if part.exists() {
            std::fs::rename(&part, &dest)?;
        }
        Ok(dest)
    }
}
