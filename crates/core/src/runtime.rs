use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("drxwnify-worker")
            .build()
            .expect("Failed to build Tokio runtime")
    })
}

pub fn spawn_async<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    get_runtime().spawn(future)
}

pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    get_runtime().block_on(future)
}
