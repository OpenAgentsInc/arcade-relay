use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{upgrade, Body, Request as HyperRequest, Response};
use std::convert::Infallible;
use std::convert::TryFrom;
use tokio_tungstenite::tungstenite::handshake::server::Request as TungsteniteRequest;
use tokio_tungstenite::WebSocketStream;
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let addr = ([127, 0, 0, 1], 8080).into();

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Server started on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

async fn handle_request(mut request: HyperRequest<Body>) -> Result<Response<Body>, Infallible> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    if let Ok(_) = req.parse(request.headers().iter().map(|(name, value)| {
        (
            name.as_str(),
            value.to_str().expect("header value should be valid"),
        )
    })) {
        let tungstenite_req = TungsteniteRequest::from_headers(
            req.method.unwrap(),
            req.path.unwrap(),
            req.version.unwrap(),
            req.headers,
        )
        .unwrap();

        match TungsteniteRequest::try_from(tungstenite_req) {
            Ok(ws_req) => {
                let response = create_response(&ws_req);
                tokio::spawn(async move {
                    match upgrade::on(&mut request).await {
                        Ok(upgraded) => {
                            let ws_stream = WebSocketStream::from_raw_socket(
                                upgraded,
                                tokio_tungstenite::tungstenite::protocol::Role::Server,
                                None,
                            )
                            .await;
                            // Handle WebSocket stream here.
                        }
                        Err(e) => eprintln!("Error upgrading connection: {}", e),
                    }
                });
                Ok(response)
            }
            Err(_) => {
                let response = Response::builder()
                    .status(400)
                    .body(Body::from("Invalid request"))
                    .unwrap();
                Ok(response)
            }
        }
    } else {
        let response = Response::builder()
            .status(400)
            .body(Body::from("Invalid request"))
            .unwrap();
        Ok(response)
    }
}

fn create_response(
    ws_req: &tokio_tungstenite::tungstenite::handshake::server::Request,
) -> Response<Body> {
    ws_req.accept().unwrap().finish().unwrap()
}
