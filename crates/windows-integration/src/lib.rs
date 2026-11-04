use drxwnify_core::{AppEdition, DrxwnifyError, DrxwnifyResult};
use tokio::net::TcpListener;

pub struct SingleInstance;

impl SingleInstance {
    pub async fn check_or_bind(edition: AppEdition) -> DrxwnifyResult<TcpListener> {
        let port = match edition {
            AppEdition::Full => 47834,
            AppEdition::Mini => 47835,
        };

        let addr = format!("127.0.0.1:{}", port);
        match TcpListener::bind(&addr).await {
            Ok(listener) => Ok(listener),
            Err(_) => {
                // Instance already running! Connect and signal activate
                use tokio::io::AsyncWriteExt;
                if let Ok(mut stream) = tokio::net::TcpStream::connect(&addr).await {
                    let _ = stream.write_all(b"show\n").await;
                }
                Err(DrxwnifyError::Windows(format!(
                    "Another instance of Drxwnify {:?} is already running",
                    edition
                )))
            }
        }
    }
}
