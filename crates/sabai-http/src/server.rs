use std::convert::Infallible;
use std::error::Error as StdError;
use std::io;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use std::time::Duration;

use http_body_util::{BodyExt, LengthLimitError, Limited};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use sabai_core::{
    App, Body, Config, Error, IntoResponse, Request, Response, Result, StatusCode, log,
};
use tokio::net::TcpListener;

use crate::catch_panic::catch_panic;
use crate::{Router, Routes};

const DEFAULT_BODY_KB: u64 = 2048;
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;

/// How much a client may send and how long it may take; the times stop slowloris clients.
#[derive(Clone, Copy)]
pub(crate) struct Limits {
    body_bytes: usize,
    headers: Duration,
    body: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self::new(
            DEFAULT_BODY_KB,
            DEFAULT_TIMEOUT_SECONDS,
            DEFAULT_TIMEOUT_SECONDS,
        )
    }
}

impl Limits {
    fn new(body_kb: u64, header_seconds: u64, body_seconds: u64) -> Self {
        Self {
            body_bytes: (body_kb * 1024) as usize,
            headers: Duration::from_secs(header_seconds),
            body: Duration::from_secs(body_seconds),
        }
    }

    fn from_config(config: &Config) -> Result<Self> {
        Ok(Self::new(
            positive(config, "app.max_body_kb", DEFAULT_BODY_KB)?,
            positive(config, "app.header_timeout", DEFAULT_TIMEOUT_SECONDS)?,
            positive(config, "app.body_timeout", DEFAULT_TIMEOUT_SECONDS)?,
        ))
    }
}

fn positive(config: &Config, key: &str, default: u64) -> Result<u64> {
    match config.get_or(key, default)? {
        0 => Err(Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "`{key}` in config/app.toml must be at least 1; check the value there or the variable it reads in .env"
            ),
        )),
        value => Ok(value),
    }
}

/// Serves `router` until the process stops, on `app.host`:`app.port` from config
/// (default `127.0.0.1:8000`), like `php artisan serve`.
pub fn serve(app: Arc<App>, router: Router) -> Result<()> {
    let routes = router.compile()?;
    routes.prepare(&app)?;
    let host: String = app.config().get_or("app.host", "127.0.0.1".to_owned())?;
    let port: u16 = app.config().get_or("app.port", 8000)?;
    let listener = StdTcpListener::bind((host.as_str(), port))
        .map_err(|error| listen_failed(error, &host, port))?;
    log::info!("Sabai is serving on http://{}", listener.local_addr()?);
    let limits = Limits::from_config(app.config())?;
    serve_on(
        listener,
        Arc::new(Site {
            app,
            routes,
            limits,
        }),
    )
}

/// What every connection needs, shared behind one `Arc`.
pub(crate) struct Site {
    app: Arc<App>,
    routes: Routes,
    limits: Limits,
}

fn listen_failed(error: io::Error, host: &str, port: u16) -> Error {
    let hint = match error.kind() {
        io::ErrorKind::AddrInUse => {
            " because another program is using that port; stop it or change `app.port` in config/app.toml"
        }
        _ => "",
    };
    Error::from(error).context(format!("could not listen on {host}:{port}{hint}"))
}

pub(crate) fn serve_on(listener: StdTcpListener, site: Arc<Site>) -> Result<()> {
    listener.set_nonblocking(true)?;
    sabai_core::block_on(accept_forever(listener, site))?
}

async fn accept_forever(listener: StdTcpListener, site: Arc<Site>) -> Result<()> {
    let listener = TcpListener::from_std(listener)?;
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(serve_connection(TokioIo::new(stream), site.clone()));
            }
            // One failed accept, such as running out of file descriptors, must not stop the server.
            Err(error) => log::warn!("could not accept a connection: {error}"),
        }
    }
}

async fn serve_connection(io: TokioIo<tokio::net::TcpStream>, site: Arc<Site>) {
    let mut http = http1::Builder::new();
    // hyper only enforces its header timeout when it has a timer; without one it is silently off.
    http.timer(TokioTimer::new())
        .header_read_timeout(site.limits.headers);
    let service = service_fn(move |request| handle(site.clone(), request));
    if let Err(error) = http.serve_connection(io, service).await {
        log::debug!("connection closed: {error}");
    }
}

async fn handle(
    site: Arc<Site>,
    request: hyper::Request<Incoming>,
) -> Result<Response, Infallible> {
    let response = match read_body(request, site.limits).await {
        Ok(request) => {
            // Handlers are caught inside the middleware already; this catches a middleware that panics.
            let dispatch = catch_panic(site.routes.dispatch(request.with_app(site.app.clone())));
            site.app.scope(dispatch).await
        }
        Err(error) => error.into_response(),
    };
    Ok(response)
}

async fn read_body(request: hyper::Request<Incoming>, limits: Limits) -> Result<Request> {
    let (parts, body) = request.into_parts();
    let collect = Limited::new(body, limits.body_bytes).collect();
    let collected = tokio::time::timeout(limits.body, collect)
        .await
        .map_err(|_| Error::from_status(StatusCode::REQUEST_TIMEOUT))?
        .map_err(body_error)?;
    let body = Body::from(collected.to_bytes());
    Ok(Request::from(http::Request::from_parts(parts, body)))
}

fn body_error(error: Box<dyn StdError + Send + Sync>) -> Error {
    if error.is::<LengthLimitError>() {
        return Error::from_status(StatusCode::PAYLOAD_TOO_LARGE);
    }
    Error::from_status(StatusCode::BAD_REQUEST)
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    use sabai_core::Config;

    use super::*;

    async fn hello() -> &'static str {
        "Hello, Sabai"
    }

    async fn current_app() -> &'static str {
        if App::current().is_ok() {
            "current"
        } else {
            "none"
        }
    }

    async fn broken() -> &'static str {
        panic!("the handler has a bug")
    }

    struct BrokenMiddleware;

    impl crate::Middleware for BrokenMiddleware {
        async fn handle(&self, _request: Request, _next: crate::Next) -> Response {
            panic!("the middleware has a bug")
        }
    }

    fn running_server() -> std::net::SocketAddr {
        running_server_with(Limits::default())
    }

    fn running_server_with(limits: Limits) -> std::net::SocketAddr {
        let listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let mut router = Router::new();
        router.get("/", hello);
        router.get("/current", current_app);
        router.get("/broken", broken);
        router
            .get("/broken-middleware", hello)
            .middleware(BrokenMiddleware);
        let site = Site {
            app: App::builder(Config::default()).boot().unwrap(),
            routes: router.compile().unwrap(),
            limits,
        };
        std::thread::spawn(move || serve_on(listener, Arc::new(site)));
        address
    }

    fn send(address: std::net::SocketAddr, head: &str, body: &[u8]) -> String {
        let mut stream = TcpStream::connect(address).unwrap();
        // A server that failed to start still accepts connections, so fail instead of hanging.
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    #[test]
    fn a_real_http_request_reaches_the_handler() {
        let response = send(
            running_server(),
            "GET / HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n",
            b"",
        );

        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        assert!(response.ends_with("Hello, Sabai"));
    }

    #[test]
    fn handlers_run_with_the_served_app_as_current() {
        let response = send(
            running_server(),
            "GET /current HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n",
            b"",
        );

        assert!(response.ends_with("current"), "{response}");
    }

    #[test]
    fn a_port_in_use_explains_how_to_fix_it() {
        let taken = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let port = taken.local_addr().unwrap().port();
        let error = StdTcpListener::bind(("127.0.0.1", port)).unwrap_err();

        let message = listen_failed(error, "127.0.0.1", port).to_string();

        let expected = format!("could not listen on 127.0.0.1:{port} because another program");
        assert!(message.starts_with(&expected), "{message}");
        assert!(message.contains("change `app.port` in config/app.toml"));
    }

    #[test]
    fn a_body_over_the_limit_is_rejected_before_any_handler_runs() {
        let body = vec![b'x'; Limits::default().body_bytes + 1];
        let head = format!(
            "POST / HTTP/1.1\r\nHost: test\r\nConnection: close\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );

        let response = send(running_server(), &head, &body);

        assert!(
            response.starts_with("HTTP/1.1 413 Payload Too Large"),
            "{response}"
        );
    }

    #[test]
    fn a_handler_that_panics_answers_500_and_the_server_keeps_serving() {
        let address = running_server();

        let response = send(
            address,
            "GET /broken HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n",
            b"",
        );
        let next = send(
            address,
            "GET / HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n",
            b"",
        );

        assert!(response.starts_with("HTTP/1.1 500"), "{response:?}");
        assert!(
            response.ends_with(r#"{"message":"Server Error"}"#),
            "{response}"
        );
        assert!(next.starts_with("HTTP/1.1 200 OK"), "{next}");
    }

    #[test]
    fn a_middleware_that_panics_answers_500_too() {
        let response = send(
            running_server(),
            "GET /broken-middleware HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n",
            b"",
        );

        assert!(response.starts_with("HTTP/1.1 500"), "{response:?}");
    }

    #[test]
    fn head_sends_the_get_headers_and_length_without_the_body() {
        let response = send(
            running_server(),
            "HEAD / HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n",
            b"",
        );

        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        assert!(response.contains("content-length: 12\r\n"), "{response}");
        assert!(response.ends_with("\r\n\r\n"), "{response:?}");
    }

    fn impatient_server() -> std::net::SocketAddr {
        running_server_with(Limits {
            headers: Duration::from_millis(300),
            body: Duration::from_millis(300),
            ..Limits::default()
        })
    }

    #[test]
    fn a_client_that_never_finishes_its_headers_is_disconnected() {
        let started = std::time::Instant::now();

        let response = send(impatient_server(), "GET / HTTP/1.1\r\nHost: te", b"");

        assert!(!response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn a_client_that_never_finishes_its_body_gets_a_408() {
        let head =
            "POST / HTTP/1.1\r\nHost: test\r\nConnection: close\r\nContent-Length: 10\r\n\r\n";

        let response = send(impatient_server(), head, b"abc");

        assert!(response.starts_with("HTTP/1.1 408"), "{response}");
    }

    fn config_with(app_toml: &str) -> Config {
        let base = std::env::temp_dir().join(format!(
            "sabai-limits-{}-{}",
            std::process::id(),
            app_toml.len()
        ));
        std::fs::create_dir_all(base.join("config")).unwrap();
        std::fs::write(base.join("config/app.toml"), app_toml).unwrap();
        Config::load(&base).unwrap()
    }

    #[test]
    fn limits_come_from_config_app_toml_and_default_when_missing() {
        // The same `${...:-default}` form as `config/app.toml`, with variables no one sets.
        let raised = config_with(concat!(
            "max_body_kb = \"${SABAI_TEST_UNSET_KB:-10240}\"\n",
            "body_timeout = \"${SABAI_TEST_UNSET_SECONDS:-120}\"\n",
        ));

        let limits = Limits::from_config(&raised).unwrap();

        assert_eq!(limits.body_bytes, 10 * 1024 * 1024);
        assert_eq!(limits.body, Duration::from_secs(120));
        assert_eq!(limits.headers, Duration::from_secs(30));
    }

    #[test]
    fn a_zero_limit_stops_the_app_and_names_the_key() {
        let error = Limits::from_config(&config_with("max_body_kb = 0\n"))
            .err()
            .unwrap()
            .to_string();

        assert!(
            error.contains("`app.max_body_kb` in config/app.toml must be at least 1"),
            "{error}"
        );
    }
}
