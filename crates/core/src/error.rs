use thiserror::Error;

#[derive(Error, Debug)]
pub enum DrxwnifyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Audio engine error: {0}")]
    Audio(String),

    #[error("Network request failed: {0}")]
    Network(String),

    #[error("Track resolution failed for {id}: {reason}")]
    Resolution { id: String, reason: String },

    #[error("Nostr protocol error: {0}")]
    Nostr(String),

    #[error("Storage corrupted or inaccessible: {0}")]
    Storage(String),

    #[error("Windows integration error: {0}")]
    Windows(String),

    #[error("Unknown error: {0}")]
    Other(String),
}

pub type DrxwnifyResult<T> = Result<T, DrxwnifyError>;
