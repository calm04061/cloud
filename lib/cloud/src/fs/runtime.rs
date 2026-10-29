use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Runtime;

static BRIDGE_RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    BRIDGE_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .expect("failed to create async bridge runtime")
        })
        .block_on(future)
}
