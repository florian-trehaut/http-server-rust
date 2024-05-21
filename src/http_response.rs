use std::fmt::Display;

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
}

impl Display for HTTPResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status)?;
        match self.header.clone() {
            Some(header) => write!(f, "{header}")?,
            None => write!(f, "\r\n")?,
        }
        match &self.body {
            Some(body) => write!(f, "{body}"),
            None => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HTTPResponseBuilder {
    status: ResponseStatus,
    header: Option<ResponseHeader>,
    body: Option<ResponseBody>,
}
impl HTTPResponseBuilder {
    pub fn with_body(&self, content: &str, content_type: ContentType) -> Self {
        let body = ResponseBody(content.to_string());
        let header = ResponseHeader::new(content_type, &body);
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
    location: Option<String>,
}
impl ResponseHeader {
    fn new(content_type: ContentType, body: &ResponseBody) -> Self {
        Self {
            content_type,
            content_length: ContentLength::from_body(body),
            location: None,
        }
    }
    const fn add_location(&self, location: String) -> Self {
        Self {
            content_type: self.content_type,
            content_length: self.content_length,
            location: Some(location),
        }
    }
}
impl Display for ResponseHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Content-Type: {}\r\n", self.content_type)?;
        write!(f, "Content-Length: {}\r\n", self.content_length)?;
        match self.location.clone() {
            Some(location) => write!(f, "Location: {location}\r\n")?,
            None => (),
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
struct ResponseBody(String);
impl ResponseBody {
    fn length(&self) -> usize {
        self.0.len()
    }
}
impl Display for ResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
