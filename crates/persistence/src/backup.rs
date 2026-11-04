use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use drxwnify_shared_models::*;
use rusqlite::Connection;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct PhoneBackupImporter;

impl PhoneBackupImporter {
    pub fn import_backup_zip(zip_path: &Path) -> DrxwnifyResult<LibraryData> {
        let file = File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| DrxwnifyError::Storage(e.to_string()))?;

        let mut library = LibraryData::default();
        let mut temp_db_path = None;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| DrxwnifyError::Storage(e.to_string()))?;
            let name = file.name().to_string();

            if name.ends_with("song.db") {
                let temp_path = std::env::temp_dir().join(format!("drxwn_import_{}.db", std::process::id()));
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                std::fs::write(&temp_path, &buffer)?;
                temp_db_path = Some(temp_path);
                break;
            }
        }

        if let Some(db_path) = temp_db_path {
            Self::parse_sqlite_songs(&db_path, &mut library)?;
            let _ = std::fs::remove_file(db_path);
        }

        Ok(library)
    }

    fn parse_sqlite_songs(db_path: &Path, library: &mut LibraryData) -> DrxwnifyResult<()> {
        let conn = Connection::open(db_path).map_err(|e| DrxwnifyError::Storage(e.to_string()))?;

        // Query song table if exists
        let mut stmt = conn
            .prepare("SELECT id, title, artist, duration, cover FROM song")
            .map_err(|e| DrxwnifyError::Storage(e.to_string()))?;

        let track_rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let artist: String = row.get(2)?;
                let duration: i64 = row.get(3).unwrap_or(0);
                let cover: Option<String> = row.get(4).ok();

                Ok(Track {
                    id,
                    title,
                    artists: vec![ArtistRef { name: artist, id: None }],
                    album: None,
                    duration_sec: duration,
                    cover,
                    explicit: false,
                    youtube_id: None,
                    isrc: None,
                    local_path: None,
                    bitrate_kbps: None,
                    source: None,
                })
            })
            .map_err(|e| DrxwnifyError::Storage(e.to_string()))?;

        for track_res in track_rows {
            if let Ok(track) = track_res {
                library.liked.push(track);
            }
        }

        Ok(())
    }
}
