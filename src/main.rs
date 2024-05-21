use anyhow::{Context, Result};
use http_server_starter_rust::client_handler::ClientHandler;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221")
        .await
        .context("Can't start listener")?;

    while let Ok((mut stream, _socket_address)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = ClientHandler::parse_request(&mut stream).await {
                panic!("Error handling client request: {e}");
            }
        });
    }

    Ok(())
}
