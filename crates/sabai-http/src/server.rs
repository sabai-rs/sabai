use std::convert::Infallible;
use std::error::Error as StdError;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;

use http_body_util::{BodyExt, LengthLimitError, Limited};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use sabai_core::{App, Body, Error, IntoResponse, Request, Response, Result, StatusCode, log};
use tokio::net::TcpListener;

use crate::{Router, Routes};

// Bodies are read in full before the handler runs, so this caps memory per request.
// Per-route limits come in M1-04.
const BODY_LIMIT: usize = 2 * 1024 * 1024;

/// Serves `router` until the process stops, on `app.host`:`app.port` from config
/// (default `127.0.0.1:8000`), like `php artisan serve`.
pub fn serve(app: Arc<App>, router: Router) -> Result<()> {
    let routes = Arc::new(router.compile()?);
    let host: String = app.config().get_or("app.host", "127.0.0.1".to_owned())?;
    let port: u16 = app.config().get_or("app.port", 8000)?;
    let listener = StdTcpListener::bind((host.as_str(), port)).map_err(|error| {
        Error::from(error).context(format!("could not listen on {host}:{port}"))
    })?;
    log::info!("Sabai is serving on http://{}", listener.local_addr()?);
    serve_on(listener, routes)
}

pub(crate) fn serve_on(listener: StdTcpListener, routes: Arc<Routes>) -> Result<()> {
    listener.set_nonblocking(true)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()?;
    runtime.block_on(accept_forever(listener, routes))
}

async fn accept_forever(listener: StdTcpListener, routes: Arc<Routes>) -> Result<()> {
    let listener = TcpListener::from_std(listener)?;
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(serve_connection(TokioIo::new(stream), routes.clone()));
            }
            // Such as running out of file descriptors: one failed accept must not stop the server.
            Err(error) => log::warn!("could not accept a connection: {error}"),
        }
    }
}

async fn serve_connection(io: TokioIo<tokio::net::TcpStream>, routes: Arc<Routes>) {
    let service = service_fn(move |request| handle(routes.clone(), request));
    if let Err(error) = http1::Builder::new().serve_connection(io, service).await {
        log::debug!("connection closed: {error}");
    }
}

async fn handle(
    routes: Arc<Routes>,
    request: hyper::Request<Incoming>,
) -> Result<Response, Infallible> {
    let response = match read_body(request).await {
        Ok(request) => routes.dispatch(request).await,
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

    use super::*;

    async fn hello() -> &'static str {
        "Hello, Sabai"
    }

    fn running_server() -> std::net::SocketAddr {
        let listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let routes = Arc::new(Router::new().get("/", hello).compile().unwrap());
        std::thread::spawn(move || serve_on(listener, routes));
        address
    }

    fn send(address: std::net::SocketAddr, head: &str, body: &[u8]) -> String {
        let mut stream = TcpStream::connect(address).unwrap();
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
}
