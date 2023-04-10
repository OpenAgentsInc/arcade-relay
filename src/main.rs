use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:9002";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Ping(data)) => {
                ws_stream
                    .send(Message::Pong(data))
                    .await
                    .expect("Failed to send pong");
            }
            Ok(Message::Close(_)) => {
                break;
            }
            _ => (),
        }
    }
}
