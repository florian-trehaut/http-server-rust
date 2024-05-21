use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HTTPResponse {
    status: Status,
    header: Option<Header>,
    body: Option<Body>,
}
impl HTTPResponse {
    pub const fn new_builder(status: Status) -> HTTPResponseBuilder {
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
    status: Status,
    header: Option<Header>,
    body: Option<Body>,
}
impl HTTPResponseBuilder {
    pub fn with_body(&self, content: &str, content_type: ContentType) -> Self {
        let body = Body(content.to_string());
        let header = Header::new(content_type, &body);
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
pub enum Status {
    Http200,
    Http404,
}
impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http200 => write!(f, "HTTP/1.1 200 OK\r\n"),
            Self::Http404 => write!(f, "HTTP/1.1 404 Not Found\r\n"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Header {
    content_type: ContentType,
    content_length: ContentLength,
}
impl Header {
    fn new(content_type: ContentType, body: &Body) -> Self {
        Self {
            content_type,
            content_length: ContentLength::from_body(body),
        }
    }
}
impl Display for Header {
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
    fn from_body(body: &Body) -> Self {
        Self(body.length())
    }
}
impl Display for ContentLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Body(String);
impl Body {
    fn length(&self) -> usize {
        self.0.len()
    }
}
impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
