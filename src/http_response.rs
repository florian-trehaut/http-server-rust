use std::fmt::Display;

use crate::{gzip::Gzip, http_request::Encoding};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HTTPResponse {
    status: ResponseStatus,
    header: Option<ResponseHeader>,
    body: Option<ResponseBody>,
}
impl HTTPResponse {
    pub const fn new_builder(status: ResponseStatus) -> HTTPResponseBuilder {
        HTTPResponseBuilder {
            status,
            header: None,
            body: None,
        }
    }
    pub fn as_http_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend_from_slice(format!("{}", self.status).as_bytes());

        match self.header.clone() {
            Some(header) => {
                if let Some(encoding) = header.content_encoding {
                    buf.extend_from_slice(format!("Content-Encoding: {encoding}\r\n").as_bytes());
                };
                buf.extend_from_slice(
                    format!("Content-Type: {}\r\n", header.content_type).as_bytes(),
                );
                buf.extend_from_slice(
                    format!("Content-Length: {}\r\n", header.content_length).as_bytes(),
                );
                if let Some(location) = header.location {
                    buf.extend_from_slice(format!("Location: {location}\r\n").as_bytes());
                }
            }
            None => buf.extend_from_slice(b"\r\n"),
        }
        buf.extend_from_slice(b"\r\n");

        match &self.body {
            Some(body) => buf.extend_from_slice(&body.0),
            None => (),
        }

        buf
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HTTPResponseBuilder {
    status: ResponseStatus,
    header: Option<ResponseHeader>,
    body: Option<ResponseBody>,
}
impl HTTPResponseBuilder {
    pub fn with_body(
        &self,
        content: &str,
        content_type: ContentType,
        encoding: &[Encoding],
    ) -> Self {
        let body = encoding.first().map_or_else(
            || ResponseBody(content.as_bytes().to_owned()),
            |_encoding| ResponseBody(Gzip::parse(content).as_bytes().to_owned()),
        );
        let header = ResponseHeader::new(content_type, &body, encoding.first().copied());
        Self {
            status: self.status,
            header: Some(header),
            body: Some(body),
        }
    }
    pub fn with_location(&self, location: String) -> Self {
        // sry im lazy today
        let header = self.header.clone().unwrap().add_location(location);
        Self {
            status: self.status,
            header: Some(header),
            body: self.body.clone(),
        }
    }
    pub fn build(&self) -> HTTPResponse {
        HTTPResponse {
            status: self.status,
            header: self.header.clone(),
            body: self.body.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// We accept to hardcode version
pub enum ResponseStatus {
    Http200,
    Http201,
    Http400,
    Http404,
    Http500,
}
impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http200 => write!(f, "HTTP/1.1 200 OK\r\n"),
            Self::Http201 => write!(f, "HTTP/1.1 201 Created\r\n"),
            Self::Http400 => write!(f, "HTTP/1.1 400 Bad Request\r\n"),
            Self::Http404 => write!(f, "HTTP/1.1 404 Not Found\r\n"),
            Self::Http500 => write!(f, "HTTP/1.1 500 Internal Server Error\r\n"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResponseHeader {
    content_type: ContentType,
    content_length: ContentLength,
    content_encoding: Option<Encoding>,
    location: Option<String>,
}
impl ResponseHeader {
    fn new(content_type: ContentType, body: &ResponseBody, encoding: Option<Encoding>) -> Self {
        Self {
            content_type,
            content_length: ContentLength::from_body(body),
            location: None,
            content_encoding: encoding,
        }
    }
    const fn add_location(&self, location: String) -> Self {
        Self {
            content_type: self.content_type,
            content_length: self.content_length,
            location: Some(location),
            content_encoding: self.content_encoding,
        }
    }
}

impl Display for ResponseHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(encoding) = self.content_encoding {
            write!(f, "Content-Encoding: {encoding}\r\n")?;
        }
        write!(f, "Content-Type: {}\r\n", self.content_type)?;
        write!(f, "Content-Length: {}\r\n", self.content_length)?;
        if let Some(location) = self.location.clone() {
            write!(f, "Location: {location}\r\n")?;
        }
        write!(f, "\r\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentType {
    TextPlain,
    OctetStream,
}
impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextPlain => write!(f, "text/plain"),
            Self::OctetStream => write!(f, "application/octet-stream"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ContentLength(usize);
impl ContentLength {
    fn from_body(body: &ResponseBody) -> Self {
        Self(body.length())
    }
}
impl Display for ContentLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ResponseBody(Vec<u8>);
impl ResponseBody {
    fn length(&self) -> usize {
        self.0.len()
    }
}
