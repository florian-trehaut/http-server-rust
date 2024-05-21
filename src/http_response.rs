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
        match self.header {
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
    pub fn build(&self) -> HTTPResponse {
        HTTPResponse {
            status: self.status,
            header: self.header,
            body: self.body.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// We accept to hardcode version
pub enum ResponseStatus {
    Http200,
    Http400,
    Http404,
}
impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http200 => write!(f, "HTTP/1.1 200 OK\r\n"),
            Self::Http400 => write!(f, "HTTP/1.1 400 Bad Request\r\n"),
            Self::Http404 => write!(f, "HTTP/1.1 404 Not Found\r\n"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ResponseHeader {
    content_type: ContentType,
    content_length: ContentLength,
}
impl ResponseHeader {
    fn new(content_type: ContentType, body: &ResponseBody) -> Self {
        Self {
            content_type,
            content_length: ContentLength::from_body(body),
        }
    }
}
impl Display for ResponseHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Content-Type: {}\r\n", self.content_type)?;
        write!(f, "Content-Length: {}\r\n", self.content_length)?;
        write!(f, "\r\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentType {
    TextPlain,
}
impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextPlain => write!(f, "text/plain"),
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
