use std::future::Future;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::{App, Error, Result, StatusCode};

tokio::task_local! {
    static CURRENT: Arc<App>;
}

impl App {
    /// The app this request, test or command runs in, like Laravel's `app()`.
    pub fn current() -> Result<Arc<App>> {
        CURRENT.try_with(Arc::clone).map_err(|_| no_current_app())
    }

    /// Runs `future` with this app as [`App::current`]; `serve`, `TestApp` and commands do this.
    pub fn scope<F: Future>(self: &Arc<Self>, future: F) -> impl Future<Output = F::Output> {
        CURRENT.scope(Arc::clone(self), future)
    }
}

/// Runs `future` in the background with the current app, like `tokio::spawn`.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    match App::current() {
        Ok(app) => tokio::spawn(CURRENT.scope(app, future)),
        Err(_) => tokio::spawn(future),
    }
}

fn no_current_app() -> Error {
    Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).context(
        "no app in this task; start background work with `sabai::spawn`, not `tokio::spawn`, \
         and run code outside a request inside `app.scope(...)`",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(future)
    }

    #[test]
    fn the_scoped_app_is_current_and_outside_the_error_says_what_to_do() {
        let app = App::builder(Config::default()).boot().unwrap();

        let inside = block_on(app.scope(async { App::current() })).unwrap();
        let outside = App::current().err().unwrap().to_string();

        assert!(Arc::ptr_eq(&app, &inside));
        assert!(outside.contains("`sabai::spawn`"), "{outside}");
    }

    #[test]
    fn sabai_spawn_carries_the_app_and_tokio_spawn_does_not() {
        let app = App::builder(Config::default()).boot().unwrap();

        let (with_sabai, with_tokio) = block_on(app.scope(async {
            let with_sabai = spawn(async { App::current().is_ok() }).await.unwrap();
            let with_tokio = tokio::spawn(async { App::current().is_ok() })
                .await
                .unwrap();
            (with_sabai, with_tokio)
        }));

        assert!(with_sabai);
        assert!(!with_tokio);
    }
}
