use drxwnify_core::{AppPaths, DrxwnifyError, DrxwnifyResult};
use drxwnify_network::HttpClient;
use drxwnify_shared_models::Track;
use std::path::PathBuf;

pub struct DownloadManager {
    paths: AppPaths,
    http: HttpClient,
}

impl DownloadManager {
    pub fn new(paths: AppPaths, http: HttpClient) -> Self {
        Self { paths, http }
    }

    pub async fn download_track(&self, track: &Track, stream_url: &str) -> DrxwnifyResult<PathBuf> {
        let safe_title = track.title.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let safe_artist = track.artist_names().replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let file_name = format!("{} - {}.mp3", safe_artist, safe_title);
        let out_path = self.paths.downloads_dir.join(file_name);

        let resp = self
            .http
            .inner()
            .get(stream_url)
            .send()
            .await
            .map_err(|e| DrxwnifyError::Network(e.to_string()))?;

        let bytes = resp.bytes().await.map_err(|e| DrxwnifyError::Network(e.to_string()))?;
        std::fs::write(&out_path, &bytes)?;

        tracing::info!("Track downloaded to {:?}", out_path);
        Ok(out_path)
    }
}
