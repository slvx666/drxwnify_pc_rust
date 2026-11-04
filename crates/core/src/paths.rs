use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEdition {
    Full,
    Mini,
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub edition: AppEdition,
    pub root_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub audio_cache_dir: PathBuf,
    pub covers_cache_dir: PathBuf,
    pub downloads_dir: PathBuf,
    pub settings_file: PathBuf,
    pub accounts_file: PathBuf,
    pub library_file: PathBuf,
    pub queue_file: PathBuf,
    pub matches_file: PathBuf,
    pub friends_file: PathBuf,
    pub chats_file: PathBuf,
    pub log_file: PathBuf,
}

impl AppPaths {
    pub fn new(edition: AppEdition) -> Self {
        let appdata = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let folder_name = match edition {
            AppEdition::Full => "DrxwnifyRust",
            AppEdition::Mini => "DrxwnifyMini",
        };

        let root_dir = appdata.join(folder_name);
        let cache_dir = root_dir.join("cache");
        let audio_cache_dir = cache_dir.join("audio");
        let covers_cache_dir = cache_dir.join("covers");

        let downloads_dir = std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join("Music").join("Drxwnify"))
            .unwrap_or_else(|_| root_dir.join("downloads"));

        Self {
            edition,
            cache_dir: cache_dir.clone(),
            audio_cache_dir,
            covers_cache_dir,
            downloads_dir,
            settings_file: root_dir.join("settings.json"),
            accounts_file: root_dir.join("accounts.json"),
            library_file: root_dir.join("library.json"),
            queue_file: root_dir.join("queue.json"),
            matches_file: root_dir.join("matches.json"),
            friends_file: root_dir.join("friends.json"),
            chats_file: root_dir.join("chats.json"),
            log_file: root_dir.join("app.log"),
            root_dir,
        }
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.audio_cache_dir)?;
        std::fs::create_dir_all(&self.covers_cache_dir)?;
        std::fs::create_dir_all(&self.downloads_dir)?;
        Ok(())
    }

    pub fn original_kotlin_appdata() -> Option<PathBuf> {
        let path = std::env::var("APPDATA").ok().map(|p| PathBuf::from(p).join("Drxwnify"));
        if let Some(ref p) = path {
            if p.exists() {
                return path;
            }
        }
        None
    }
}
