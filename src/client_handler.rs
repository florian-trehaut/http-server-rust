use std::fs;

use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    http_request::{
        HTTPRequestLineError, RequestBody, RequestBodyError, RequestHeader, RequestHeaderError,
        RequestLine, RequestMethod,
    },
    http_response::{ContentType, HTTPResponse, ResponseStatus},
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
    /// Returns an error of type `ClientHandlerError` if the request is too large, the stream cannot be read, the request line is empty, or the request cannot be decoded to UTF-8.
    pub async fn parse_request(
        stream: &mut TcpStream,
        directory: Option<String>,
    ) -> Result<HTTPResponse, ClientHandlerError> {
        let mut buf = [0; 4096];
        let n = stream.read(&mut buf).await?;
        if n == buf.len() {
            return Err(ClientHandlerError::RequestTooLarge);
        }
        let buf = std::str::from_utf8(&buf[..n]).map_err(|e| {
            ClientHandlerError::Utf8Error(e, String::from_utf8_lossy(&buf).to_string())
        })?;
        let mut request = buf.lines();
        let Some(request_line) = request.next() else {
            return Err(ClientHandlerError::NoRequestLineFound);
        };
        let request_line: RequestLine = request_line.parse()?;
        let request_header: RequestHeader = buf.parse()?;
        let reponse = match request_line.method() {
            RequestMethod::Get => {
                println!("Get command received");
                Self::get(stream, buf, request_line, request_header, directory).await?
            }
            RequestMethod::Post => {
                println!("Post command received : {buf}");
                Self::post(stream, buf, request_line, request_header, directory).await?
            }
        };
        Ok(reponse)
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
        request_line: RequestLine,
        request_header: RequestHeader,
        directory: Option<String>,
    ) -> Result<HTTPResponse, ClientHandlerError> {
        let path = request_line.path().to_string();
        match path.as_str() {
            "/" => Ok(Self::respond(
                stream,
                HTTPResponse::new_builder(ResponseStatus::Http200).build(),
                request,
            )
            .await?),
            _ if path.starts_with("/echo/") => {
                let content = path.split('/').nth(2).unwrap_or_default();
                let response = HTTPResponse::new_builder(ResponseStatus::Http200)
                    .with_body(
                        content,
                        ContentType::TextPlain,
                        request_header.accept_encoding(),
                    )
                    .build();
                Ok(Self::respond(stream, response, request).await?)
            }
            _ if path.starts_with("/user-agent") => {
                let Some(user_agent) = request_header.user_agent() else {
                    {
                        let response = HTTPResponse::new_builder(ResponseStatus::Http400)
                            .with_body(
                                "Missing User-Agent header",
                                ContentType::TextPlain,
                                request_header.accept_encoding(),
                            )
                            .build();
                        return Self::respond(stream, response, request).await;
                    }
                };
                let response = HTTPResponse::new_builder(ResponseStatus::Http200)
                    .with_body(
                        &user_agent.to_string(),
                        ContentType::TextPlain,
                        request_header.accept_encoding(),
                    )
                    .build();
                Ok(Self::respond(stream, response, request).await?)
            }
            _ if path.starts_with("/files/") => match path.get("/files/".len()..) {
                Some(filepath) if !filepath.is_empty() => {
                    let Some(directory) = directory else {
                        let response = HTTPResponse::new_builder(ResponseStatus::Http404).build();
                        return Self::respond(stream, response, request).await;
                    };
                    let Ok(file_content) = fs::read_to_string(format!("{directory}/{filepath}"))
                    else {
                        return Self::respond(
                            stream,
                            HTTPResponse::new_builder(ResponseStatus::Http404).build(),
                            request,
                        )
                        .await;
                    };
                    let response = HTTPResponse::new_builder(ResponseStatus::Http200)
                        .with_body(
                            &file_content,
                            ContentType::OctetStream,
                            request_header.accept_encoding(),
                        )
                        .build();
                    Ok(Self::respond(stream, response, request).await?)
                }
                _ => {
                    let response = HTTPResponse::new_builder(ResponseStatus::Http400)
                        .with_body(
                            "File asked but no filename provided",
                            ContentType::TextPlain,
                            request_header.accept_encoding(),
                        )
                        .build();
                    Ok(Self::respond(stream, response, request).await?)
                }
            },
            _ => Ok(Self::respond(
                stream,
                HTTPResponse::new_builder(ResponseStatus::Http404).build(),
                request,
            )
            .await?),
        }
    }

    async fn post(
        stream: &mut TcpStream,
        request: &str,
        request_line: RequestLine,
        request_header: RequestHeader,
        directory: Option<String>,
    ) -> Result<HTTPResponse, ClientHandlerError> {
        let path = request_line.path().to_string();
        if path.starts_with("/files/") {
            match path.get("/files/".len()..) {
                Some(filepath) if !filepath.is_empty() => {
                    let Some(directory) = directory else {
                        println!("File path found in request but no directory provided in main");
                        let response = HTTPResponse::new_builder(ResponseStatus::Http404).build();
                        return Self::respond(stream, response, request).await;
                    };
                    println!("File path found and trying to write in file {directory}/{filepath}");
                    let content: RequestBody = request.parse()?;
                    let Ok(()) = fs::write(format!("{directory}/{filepath}"), content.to_string())
                    else {
                        return Self::respond(
                            stream,
                            HTTPResponse::new_builder(ResponseStatus::Http500)
                                .with_body(
                                    "Failed to write file",
                                    ContentType::TextPlain,
                                    request_header.accept_encoding(),
                                )
                                .build(),
                            request,
                        )
                        .await;
                    };
                    let response = HTTPResponse::new_builder(ResponseStatus::Http201)
                        .with_body(
                            "Resource created successfully",
                            ContentType::TextPlain,
                            request_header.accept_encoding(),
                        )
                        .with_location(format!("{directory}/{filepath}"))
                        .build();
                    Self::respond(stream, response, request).await
                }
                _ => {
                    Self::respond(
                        stream,
                        HTTPResponse::new_builder(ResponseStatus::Http400)
                            .with_body(
                                "No filepath specified",
                                ContentType::TextPlain,
                                request_header.accept_encoding(),
                            )
                            .build(),
                        request,
                    )
                    .await
                }
            }
        } else {
            println!("'{path}' is not found");
            Self::respond(
                stream,
                HTTPResponse::new_builder(ResponseStatus::Http404).build(),
                request,
            )
            .await
        }
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
    ) -> Result<HTTPResponse, ClientHandlerError> {
        println!(
            "Responding with '{}'",
            String::from_utf8_lossy(&response.as_http_bytes())
        );
        stream
            .write_all(&response.as_http_bytes())
            .await
            .map_err(|e| ClientHandlerError::ClientUnreachable(e, request.to_string()))?;

        Ok(response)
    }
}

#[derive(Debug, Error)]
#[allow(clippy::module_name_repetitions)]
pub enum ClientHandlerError {
    #[error("Request has no request line")]
    NoRequestLineFound,
    #[error("Stream cannot be read")]
    UnreadableStream(#[from] std::io::Error),
    #[error("Request line is empty")]
    EmptyRequestLine,
    #[error("Can't respond to client to request : '{1}'\r\n{0} ")]
    ClientUnreachable(tokio::io::Error, String),
    #[error("Can't decode request to Utf8 : '{1}'\r\n{0}")]
    Utf8Error(std::str::Utf8Error, String),
    #[error("Request is larger than the maximum buffer size")]
    RequestTooLarge,
    #[error("Error handling GET command: {0}")]
    GetCommandError(#[from] GetCommandError),
    #[error("{0}")]
    HTTPRequestLineError(#[from] HTTPRequestLineError),
    #[error("{0}")]
    RequestHeaderError(#[from] RequestHeaderError),
    #[error("{0}")]
    RequestBodyError(#[from] RequestBodyError),
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
        let response = ClientHandler::parse_request(&mut stream, None)
            .await
            .unwrap();
        assert_eq!(response.as_http_bytes(), b"HTTP/1.1 200 OK\r\n\r\n");
    }

    #[tokio::test]
    async fn test_parse_request_too_large() {
        let request = &b"A".repeat(4097); // 4097 bytes
        let mut stream = setup_fake_client(request).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream, None).await,
            Err(ClientHandlerError::RequestTooLarge)
        ));
    }

    #[tokio::test]
    async fn test_parse_request_no_request_line() {
        let request = b"";
        let mut stream = setup_fake_client(request).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream, None).await,
            Err(ClientHandlerError::NoRequestLineFound)
        ));
    }

    #[tokio::test]
    async fn test_parse_request_empty_request_line() {
        let request = "\r\n";
        let mut stream = setup_fake_client(request.as_bytes()).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream, None).await,
            Err(ClientHandlerError::HTTPRequestLineError(_))
        ));
    }

    #[tokio::test]
    async fn test_parse_request_invalid_utf8() {
        // Invalid UTF-8 sequence
        let request = &[0x80, 0x80, 0x80, 0x80];
        let mut stream = setup_fake_client(request).await;
        assert!(matches!(
            ClientHandler::parse_request(&mut stream, None).await,
            Err(ClientHandlerError::Utf8Error(..))
        ));
    }

    #[tokio::test]
    async fn test_get() {
        let request = "GET / HTTP/1.1\r\n\r\n";
        let request_line: RequestLine = request.parse().unwrap();
        let mut stream = setup_fake_client(request.as_bytes()).await;
        let response = ClientHandler::get(
            &mut stream,
            request,
            request_line,
            RequestHeader::_empty(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(response.as_http_bytes(), b"HTTP/1.1 200 OK\r\n\r\n");
    }

    #[tokio::test]
    async fn test_respond() {
        let request = b"GET / HTTP/1.1\r\n\r\n";
        let mut stream = setup_fake_client(request).await;
        assert!(ClientHandler::respond(
            &mut stream,
            HTTPResponse::new_builder(ResponseStatus::Http200).build(),
            std::str::from_utf8(request).unwrap()
        )
        .await
        .is_ok());
    }
    #[tokio::test]
    async fn test_get_echo() {
        let request = "GET /echo/test HTTP/1.1\r\n\r\n";
        let request_line: RequestLine = request.parse().unwrap();
        let mut stream = setup_fake_client(request.as_bytes()).await;
        let response = ClientHandler::get(
            &mut stream,
            request,
            request_line,
            RequestHeader::_empty(),
            None,
        )
        .await
        .unwrap();

        assert_eq!(
            response.as_http_bytes(),
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\ntest"
        );
    }

    #[tokio::test]
    async fn test_get_user_agent() {
        let request = "GET /user-agent HTTP/1.1\r\nUser-Agent: Test\r\n\r\n";
        let request_line: RequestLine = request.parse().unwrap();
        let request_header: RequestHeader = request.parse().unwrap();
        let mut stream = setup_fake_client(request.as_bytes()).await;
        let response = ClientHandler::get(&mut stream, request, request_line, request_header, None)
            .await
            .unwrap();
        assert_eq!(
            response.as_http_bytes(),
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\n\r\nTest"
        );
    }

    #[tokio::test]
    async fn test_get_user_agent_missing() {
        let request = "GET /user-agent HTTP/1.1\r\n\r\n";
        let request_line: RequestLine = request.parse().unwrap();
        let mut stream = setup_fake_client(request.as_bytes()).await;
        let response = ClientHandler::get(
            &mut stream,
            request,
            request_line,
            RequestHeader::_empty(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(
            response.as_http_bytes(),
            b"HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: 25\r\n\r\nMissing User-Agent header"
        );
    }

    #[tokio::test]
    async fn test_get_unknown_path() {
        let request = "GET /unknown HTTP/1.1\r\n\r\n";
        let request_line: RequestLine = request.parse().unwrap();
        let mut stream = setup_fake_client(request.as_bytes()).await;
        let response = ClientHandler::get(
            &mut stream,
            request,
            request_line,
            RequestHeader::_empty(),
            None,
        )
        .await
        .unwrap();
        assert_eq!(response.as_http_bytes(), b"HTTP/1.1 404 Not Found\r\n\r\n");
    }
}
