use drxwnify_shared_models::Track;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::ClientOptions;

pub struct DiscordRpc {
    client_id: String,
}

impl DiscordRpc {
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
        }
    }

    pub async fn update_presence(&self, track: &Track, is_playing: bool) {
        if !is_playing {
            return;
        }

        for i in 0..10 {
            let pipe_name = format!(r"\\.\pipe\discord-ipc-{}", i);
            if let Ok(mut pipe) = ClientOptions::new().open(&pipe_name) {
                let payload = json!({
                    "cmd": "SET_ACTIVITY",
                    "args": {
                        "pid": std::process::id(),
                        "activity": {
                            "details": track.title,
                            "state": track.artist_names(),
                            "assets": {
                                "large_image": "drxwnify_logo",
                                "large_text": "Drxwnify Desktop"
                            }
                        }
                    },
                    "nonce": "1"
                });

                let msg = payload.to_string();
                let op = 1u32; // FRAME
                let len = msg.len() as u32;

                let mut buf = Vec::with_capacity(8 + msg.len());
                buf.extend_from_slice(&op.to_le_bytes());
                buf.extend_from_slice(&len.to_le_bytes());
                buf.extend_from_slice(msg.as_bytes());

                let _ = pipe.write_all(&buf).await;
                break;
            }
        }
    }
}
