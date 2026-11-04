use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging(log_dir: &Path, file_name: &str) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = tracing_appender::rolling::never(log_dir, file_name);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,drxwnify=debug"));

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking);

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init();

    Some(guard)
}
