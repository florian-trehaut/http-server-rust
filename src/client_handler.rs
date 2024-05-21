use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    http_request::{HTTPHeader, HTTPHeaderError, HTTPMethod},
    http_response::{ContentType, HTTPResponse, Status},
};

/// The `ClientHandler` struct represents a handler for client connections.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientHandler;
impl ClientHandler {
    /// Parses the incoming request from the client.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to the `TcpStream` representing the client connection.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request is successfully parsed, or an error of type `ClientHandlerError` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error of type `ClientHandlerError` if the request is too large, the stream cannot be read, the header is empty, or the request cannot be decoded to UTF-8.
    pub async fn parse_request(stream: &mut TcpStream) -> Result<(), ClientHandlerError> {
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf).await?;
        if n == buf.len() {
            return Err(ClientHandlerError::RequestTooLarge);
        }
        let buf = std::str::from_utf8(&buf[..n]).map_err(|e| {
            ClientHandlerError::Utf8Error(e, String::from_utf8_lossy(&buf).to_string())
        })?;
        let mut request = buf.lines();
        let Some(header) = request.next() else {
            return Err(ClientHandlerError::NoHeaderFound);
        };
        let header: HTTPHeader = header.parse()?;
        match header.method() {
            HTTPMethod::Get => Self::get(stream, buf, header).await?,
        }
        Ok(())
    }

    /// Handles the GET request from the client.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to the `TcpStream` representing the client connection.
    /// * `request` - The parsed request string.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the GET request is successfully handled, or an error of type `ClientHandlerError` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error of type `ClientHandlerError::ClientUnreachable` if the response cannot be sent to the client.
    async fn get(
        stream: &mut TcpStream,
        request: &str,
        header: HTTPHeader,
    ) -> Result<(), ClientHandlerError> {
        let path = header.path().to_string();
        match path.as_str() {
            "/" => {
                Self::respond(
                    stream,
                    HTTPResponse::new_builder(Status::Http200).build(),
                    request,
                )
                .await?;
            }
            _ if path.starts_with("/echo/") => {
                let content = path.split('/').nth(2).unwrap_or_default();
                let response = HTTPResponse::new_builder(Status::Http200)
                    .with_body(content, ContentType::TextPlain)
                    .build();
                Self::respond(stream, response, request).await?;
            }
            _ => {
                Self::respond(
                    stream,
                    HTTPResponse::new_builder(Status::Http404).build(),
                    request,
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Sends the response to the client.
    ///
    /// # Arguments
    ///
    /// * `stream` - A mutable reference to the `TcpStream` representing the client connection.
    /// * `response` - The response string to send to the client.
    /// * `request` - The original request string.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the response is successfully sent, or an error of type `ClientHandlerError` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error of type `ClientHandlerError::ClientUnreachable` if the response cannot be sent to the client.
    async fn respond(
        stream: &mut TcpStream,
        response: HTTPResponse,
        request: &str,
    ) -> Result<(), ClientHandlerError> {
        println!("Responding with '{response}'");
        stream
            .write_all(response.to_string().as_bytes())
            .await
            .map_err(|e| ClientHandlerError::ClientUnreachable(e, request.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
#[allow(clippy::module_name_repetitions)]
pub enum ClientHandlerError {
    #[error("Request has no header")]
    NoHeaderFound,
    #[error("Stream cannot be read")]
    UnreadableStream(#[from] std::io::Error),
    #[error("Header request is empty")]
    EmptyHeader,
    #[error("Can't respond to client to request : '{1}'\r\n{0} ")]
    ClientUnreachable(tokio::io::Error, String),
    #[error("Can't decode request to Utf8 : '{1}'\r\n{0}")]
    Utf8Error(std::str::Utf8Error, String),
    #[error("Request is larger than the maximum buffer size")]
    RequestTooLarge,
    #[error("Error handling GET command: {0}")]
    GetCommandError(#[from] GetCommandError),
    #[error("{0}")]
    HTTPHeaderError(#[from] HTTPHeaderError),
}

#[derive(Error, Debug)]
pub enum GetCommandError {
    #[error("HTTP get command missing path")]
    MissingPath,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    async fn setup_fake_client(request: &[u8]) -> TcpStream {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let cloned_request = request.to_owned(); // Clone the request string
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            socket.write_all(&cloned_request).await.unwrap(); // Use the cloned request string
        });

        TcpStream::connect(addr).await.unwrap()
    }

    #[tokio::test]
    async fn test_parse_request_valid() {
        let request = b"GET / HTTP/1.1\r\n\r\n";
        let mut stream = setup_fake_client(request).await;
        assert!(ClientHandler::parse_request(&mut stream).await.is_ok());
    }

    #[tokio::test]
    async fn test_parse_request_too_large() {
        let request = &b"A".repeat(4097); // 4097 bytes
        let mut stream = setup_fake_client(request).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream).await,
            Err(ClientHandlerError::RequestTooLarge)
        ));
    }

    #[tokio::test]
    async fn test_parse_request_no_header() {
        let request = b"";
        let mut stream = setup_fake_client(request).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream).await,
            Err(ClientHandlerError::NoHeaderFound)
        ));
    }

    #[tokio::test]
    async fn test_parse_request_empty_header() {
        let request = "\r\n";
        let mut stream = setup_fake_client(request.as_bytes()).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream).await,
            Err(ClientHandlerError::HTTPHeaderError(_))
        ));
    }

    #[tokio::test]
    async fn test_parse_request_invalid_utf8() {
        // Invalid UTF-8 sequence
        let request = &[0x80, 0x80, 0x80, 0x80];
        let mut stream = setup_fake_client(request).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream).await,
            Err(ClientHandlerError::Utf8Error(..))
        ));
    }

    #[tokio::test]
    async fn test_get() {
        let request = "GET / HTTP/1.1\r\n\r\n";
        let header: HTTPHeader = request.parse().unwrap();
        let mut stream = setup_fake_client(request.as_bytes()).await;
        assert!(ClientHandler::get(&mut stream, request, header)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_respond() {
        let request = b"GET / HTTP/1.1\r\n\r\n";
        let mut stream = setup_fake_client(request).await;
        assert!(ClientHandler::respond(
            &mut stream,
            HTTPResponse::new_builder(Status::Http200).build(),
            std::str::from_utf8(request).unwrap()
        )
        .await
        .is_ok());
    }
}
