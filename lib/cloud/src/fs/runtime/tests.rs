use super::*;
use std::panic::AssertUnwindSafe;

/// 同步上下文（FUSE 回调线程）中可正常使用，且重复调用复用同一 runtime。
#[test]
fn block_on_runs_future_on_shared_bridge_runtime() {
    let value = block_on(async { 21 * 2 });
    assert_eq!(value, 42);

    // 第二次调用不得重新初始化 runtime（OnceLock）
    let value = block_on(async { value + 1 });
    assert_eq!(value, 43);
}

/// 异步上下文中调用 bridge 属于编程错误，必须 panic（否则会产生 nested runtime）。
#[tokio::test]
async fn block_on_panics_inside_async_context() {
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| block_on(async { 1 })));
    assert!(
        result.is_err(),
        "block_on must panic when called from inside an async runtime"
    );
}
