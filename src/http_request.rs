use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestHeader {
    host: Option<Host>,
    user_agent: Option<UserAgent>,
}
impl RequestHeader {
    pub const fn _host(&self) -> Option<&Host> {
        self.host.as_ref()
    }
    pub const fn user_agent(&self) -> Option<&UserAgent> {
        self.user_agent.as_ref()
    }
    pub const fn _empty() -> Self {
        Self {
            host: None,
            user_agent: None,
        }
    }
}
impl FromStr for RequestHeader {
    type Err = RequestHeaderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let host = s
            .lines()
            .find(|l| l.starts_with("Host: "))
            .map(|l| Host(l["Host: ".len()..].to_string()));
        let host = match host {
            Some(host) if host.0.is_empty() => return Err(RequestHeaderError::InvalidHost),
            Some(host) => Some(host),
            None => None,
        };
        let user_agent = s
            .lines()
            .find(|l| l.starts_with("User-Agent: "))
            .map(|l| UserAgent(l["User-Agent: ".len()..].to_string()));
        let user_agent = match user_agent {
            Some(user_agent) if user_agent.0.is_empty() => {
                return Err(RequestHeaderError::InvalidUserAgent)
            }
            Some(user_agent) => Some(user_agent),
            None => None,
        };
        Ok(Self { host, user_agent })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum RequestHeaderError {
    #[error("'Host: ' is found in HTTP request but seems empty")]
    InvalidHost,
    #[error("'User-Agent: ' is found in HTTP request but seems empty")]
    InvalidUserAgent,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Host(String);
impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserAgent(String);
impl Display for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestLine {
    method: RequestMethod,
    path: RequestPath,
    version: RequestVersion,
}
impl RequestLine {
    pub const fn method(&self) -> &RequestMethod {
        &self.method
    }

    pub const fn path(&self) -> &RequestPath {
        &self.path
    }
}
impl FromStr for RequestLine {
    type Err = HTTPRequestLineError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut request_line = s.split_whitespace();
        let Some(method) = request_line.next() else {
            return Err(HTTPRequestLineError::MissingMethod(s.to_string()));
        };
        let Some(path) = request_line.next() else {
            return Err(HTTPRequestLineError::MissingPath(s.to_string()));
        };
        if !path.starts_with('/') {
            return Err(HTTPRequestLineError::MissingPath(s.to_string()));
        }

        let Some(version) = request_line.next() else {
            return Err(HTTPRequestLineError::MissingVersion(s.to_string()));
        };
        Ok(Self {
            method: method.parse()?,
            path: path.parse()?,
            version: version.parse()?,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestMethod {
    Get,
}
impl Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
        }
    }
}
impl FromStr for RequestMethod {
    type Err = HTTPMethodError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(Self::Get),
            invalid_command => Err(HTTPMethodError::InvalidHTTPMethod(
                invalid_command.to_string(),
            )),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum HTTPMethodError {
    #[error("'{0}' is not a valid HTTP method")]
    InvalidHTTPMethod(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum HTTPRequestLineError {
    #[error("'{0}' HTTP request line has no method")]
    MissingMethod(String),
    #[error("'{0}' HTTP request line has no path")]
    MissingPath(String),
    #[error("'{0}' HTTP request line has no version")]
    MissingVersion(String),
    #[error("{0}")]
    HTTPMethodError(#[from] HTTPMethodError),
    #[error("{0}")]
    HTTPPathError(#[from] HTTPPathError),
    #[error("{0}")]
    HTTPVersionError(#[from] HTTPVersionError),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestPath(String);
impl FromStr for RequestPath {
    type Err = HTTPPathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('/') {
            return Err(HTTPPathError::InvalidHTTPPath(format!(
                "Path '{s}' does not start with '/'"
            )));
        }
        Ok(Self(s.to_string()))
    }
}
impl Display for RequestPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum HTTPPathError {
    #[error("Invalid HTTP Path :'{0}'")]
    InvalidHTTPPath(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestVersion(String);
impl FromStr for RequestVersion {
    type Err = HTTPVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("HTTP/") {
            return Err(HTTPVersionError::InvalidHTTPVersionFormat(s.to_string()));
        }
        let version = match s.trim().get("HTTP/".len()..) {
            Some(version) if !version.is_empty() => version,
            _ => return Err(HTTPVersionError::MissingVersionNumber(s.to_string())),
        };
        Ok(Self(version.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum HTTPVersionError {
    #[error("Invalid HTTP Version format, missing HTTP/ prefix : '{0}'")]
    InvalidHTTPVersionFormat(String),
    #[error("Missing HTTP Version number: '{0}'")]
    MissingVersionNumber(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_request_header_from_valid_str() {
        let request_str = "GET / HTTP/1.1\r\nHost: example.com\r\nUser-Agent: TestAgent\r\n\r\n";
        let header = RequestHeader::from_str(request_str).unwrap();
        assert_eq!(header._host().unwrap().0, "example.com");
        assert_eq!(header.user_agent().unwrap().0, "TestAgent");
    }

    #[test]
    fn test_request_header_from_str_without_host() {
        let request_str = "GET / HTTP/1.1\r\nUser-Agent: TestAgent\r\n\r\n";
        let header = RequestHeader::from_str(request_str).unwrap();
        assert!(header._host().is_none());
        assert_eq!(header.user_agent().unwrap().0, "TestAgent");
    }

    #[test]
    fn test_request_header_from_str_without_user_agent() {
        let request_str = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let header = RequestHeader::from_str(request_str).unwrap();
        assert_eq!(header._host().unwrap().0, "example.com");
        assert!(header.user_agent().is_none());
    }

    #[test]
    fn test_request_header_from_str_with_empty_host() {
        let request_str = "GET / HTTP/1.1\r\nHost: \r\nUser-Agent: TestAgent\r\n\r\n";
        let result = RequestHeader::from_str(request_str);
        assert!(matches!(result, Err(RequestHeaderError::InvalidHost)));
    }

    #[test]
    fn test_request_header_from_str_with_empty_user_agent() {
        let request_str = "GET / HTTP/1.1\r\nHost: example.com\r\nUser-Agent: \r\n\r\n";
        let result = RequestHeader::from_str(request_str);
        assert!(matches!(result, Err(RequestHeaderError::InvalidUserAgent)));
    }

    #[test]
    fn test_request_line_from_valid_str() {
        let request_str = "GET / HTTP/1.1";
        let request_line = RequestLine::from_str(request_str).unwrap();
        assert_eq!(request_line.method(), &RequestMethod::Get);
        assert_eq!(request_line.path().0, "/");
        assert_eq!(request_line.version.0, "1.1");
    }

    #[test]
    fn test_request_line_from_str_with_invalid_method() {
        let request_str = "INVALID / HTTP/1.1";
        let result = RequestLine::from_str(request_str);
        assert!(matches!(
            result,
            Err(HTTPRequestLineError::HTTPMethodError(
                HTTPMethodError::InvalidHTTPMethod(_)
            ))
        ));
    }

    #[test]
    fn test_request_line_from_str_with_missing_path() {
        let request_str = "GET HTTP/1.1";
        let result = RequestLine::from_str(request_str);
        assert!(matches!(result, Err(HTTPRequestLineError::MissingPath(_))));
    }

    #[test]
    fn test_request_line_from_str_with_missing_version() {
        let request_str = "GET /";
        let result = RequestLine::from_str(request_str);
        assert!(matches!(
            result,
            Err(HTTPRequestLineError::MissingVersion(_))
        ));
    }

    #[test]
    fn test_request_path_from_valid_str() {
        let path_str = "/test/path";
        let path = RequestPath::from_str(path_str).unwrap();
        assert_eq!(path.0, "/test/path");
    }

    #[test]
    fn test_request_path_from_str_without_leading_slash() {
        let path_str = "test/path";
        let result = RequestPath::from_str(path_str);
        assert!(matches!(result, Err(HTTPPathError::InvalidHTTPPath(_))));
    }

    #[test]
    fn test_request_version_from_valid_str() {
        let version_str = "HTTP/1.1";
        let version = RequestVersion::from_str(version_str).unwrap();
        assert_eq!(version.0, "1.1");
    }

    #[test]
    fn test_request_version_from_str_without_http_prefix() {
        let version_str = "1.1";
        let result = RequestVersion::from_str(version_str);
        assert!(matches!(
            result,
            Err(HTTPVersionError::InvalidHTTPVersionFormat(_))
        ));
    }

    #[test]
    fn test_request_version_from_str_without_version_number() {
        let version_str = "HTTP/";
        let result = RequestVersion::from_str(version_str);
        assert!(matches!(
            result,
            Err(HTTPVersionError::MissingVersionNumber(_))
        ));
    }
    #[test]
    fn test_display_host() {
        let host = Host("example.com".to_string());
        assert_eq!(format!("{host}"), "example.com");
    }

    #[test]
    fn test_display_user_agent() {
        let user_agent = UserAgent("TestAgent".to_string());
        assert_eq!(format!("{user_agent}"), "TestAgent");
    }

    #[test]
    fn test_display_request_method() {
        let method = RequestMethod::Get;
        assert_eq!(format!("{method}"), "GET");
    }

    #[test]
    fn test_display_request_path() {
        let path = RequestPath("/test/path".to_string());
        assert_eq!(format!("{path}"), "/test/path");
    }
}
