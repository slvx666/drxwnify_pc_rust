use drxwnify_core::{DrxwnifyError, DrxwnifyResult};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct Store;

impl Store {
    pub fn load_or_default<T: DeserializeOwned + Default>(path: &Path) -> T {
        if !path.exists() {
            return T::default();
        }

        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse JSON from {:?}: {e}, using default", path);
                T::default()
            }),
            Err(e) => {
                tracing::warn!("Failed to read file {:?}: {e}, using default", path);
                T::default()
            }
        }
    }

    pub fn save_atomic<T: Serialize>(path: &Path, data: &T) -> DrxwnifyResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let tmp_path = path.with_extension("tmp");
        let file = File::create(&tmp_path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, data).map_err(DrxwnifyError::Json)?;

        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }
}
