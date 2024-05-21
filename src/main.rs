use std::env;

use anyhow::{Context, Result};
use http_server_starter_rust::client_handler::ClientHandler;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let directory = env::args()
        .rposition(|arg| arg == "--directory")
        .map(|dir_arg_position| {
            env::args()
                .nth(dir_arg_position + 1)
                .expect("Directory argument given but no directory given")
        });
    if let Some(dir) = directory.clone() {
        let _ = tokio::fs::read_dir(dir)
            .await
            .expect("Can't read directory provided");
    }
    let listener = TcpListener::bind("127.0.0.1:4221")
        .await
        .context("Can't start listener")?;

    while let Ok((mut stream, _socket_address)) = listener.accept().await {
        let directory = directory.clone();
        tokio::spawn(async move {
            if let Err(e) = ClientHandler::parse_request(&mut stream, directory.clone()).await {
                panic!("Error handling client request: {e}");
            }
        });
    }

    Ok(())
}
