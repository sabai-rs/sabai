use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::{Builder, Runtime};

use crate::Result;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// The process's one tokio runtime, started on first use. `serve` and every command run on it,
/// so a database pool opened by one still works in the next.
pub fn runtime() -> Result<&'static Runtime> {
    if let Some(runtime) = RUNTIME.get() {
        return Ok(runtime);
    }
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    Ok(RUNTIME.get_or_init(|| runtime))
}

/// Runs `future` to completion on [`runtime`].
pub fn block_on<F: Future>(future: F) -> Result<F::Output> {
    Ok(runtime()?.block_on(future))
}
