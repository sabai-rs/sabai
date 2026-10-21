use std::convert::Infallible;
use std::error::Error as StdError;
use std::io;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;

use http_body_util::{BodyExt, LengthLimitError, Limited};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use sabai_core::{App, Body, Error, IntoResponse, Request, Response, Result, StatusCode, log};
use tokio::net::TcpListener;

use crate::catch_panic::catch_panic;
use crate::{Router, Routes};

// Bodies are buffered before the handler runs, so this caps memory per request.
const BODY_LIMIT: usize = 2 * 1024 * 1024;

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
    serve_on(listener, Arc::new(Site { app, routes }))
}

/// What every connection needs, shared behind one `Arc`.
pub(crate) struct Site {
    app: Arc<App>,
    routes: Routes,
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
    let service = service_fn(move |request| handle(site.clone(), request));
    if let Err(error) = http1::Builder::new().serve_connection(io, service).await {
        log::debug!("connection closed: {error}");
    }
}

async fn handle(
    site: Arc<Site>,
    request: hyper::Request<Incoming>,
) -> Result<Response, Infallible> {
    let response = match read_body(request).await {
        Ok(request) => {
            // Handlers are caught inside the middleware already; this catches a middleware that panics.
            let dispatch = catch_panic(site.routes.dispatch(request.with_app(site.app.clone())));
            site.app.scope(dispatch).await
        }
        Err(error) => error.into_response(),
    };
    Ok(response)
}

async fn read_body(request: hyper::Request<Incoming>) -> Result<Request> {
    let (parts, body) = request.into_parts();
    let collected = Limited::new(body, BODY_LIMIT)
        .collect()
        .await
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
        let body = vec![b'x'; BODY_LIMIT + 1];
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
}
