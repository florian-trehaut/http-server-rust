use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HTTPHeader {
    method: HTTPMethod,
    path: HTTPPath,
    version: HTTPVersion,
}
impl HTTPHeader {
    pub const fn method(&self) -> &HTTPMethod {
        &self.method
    }

    pub const fn path(&self) -> &HTTPPath {
        &self.path
    }
}
impl FromStr for HTTPHeader {
    type Err = HTTPHeaderError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut header = s.split_whitespace();
        let Some(method) = header.next() else {
            return Err(HTTPHeaderError::MissingMethod(s.to_string()));
        };
        let Some(path) = header.next() else {
            return Err(HTTPHeaderError::MissingPath(s.to_string()));
        };
        let Some(version) = header.next() else {
            return Err(HTTPHeaderError::MissingVersion(s.to_string()));
        };
        Ok(Self {
            method: method.parse()?,
            path: path.parse()?,
            version: version.parse()?,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HTTPMethod {
    Get,
}
impl Display for HTTPMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
        }
    }
}
impl FromStr for HTTPMethod {
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
pub enum HTTPHeaderError {
    #[error("'{0}' HTTP header has no method")]
    MissingMethod(String),
    #[error("'{0}' HTTP header has no path")]
    MissingPath(String),
    #[error("'{0}' HTTP header has no version")]
    MissingVersion(String),
    #[error("{0}")]
    HTTPMethodError(#[from] HTTPMethodError),
    #[error("{0}")]
    HTTPPathError(#[from] HTTPPathError),
    #[error("{0}")]
    HTTPVersionError(#[from] HTTPVersionError),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HTTPPath(String);
impl FromStr for HTTPPath {
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
impl Display for HTTPPath {
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
pub struct HTTPVersion(String);
impl FromStr for HTTPVersion {
    type Err = HTTPVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("HTTP/") {
            return Err(HTTPVersionError::InvalidHTTPVersionFormat(s.to_string()));
        }
        let Some(version) = s.get(4..) else {
            return Err(HTTPVersionError::MissingVersionNumber(s.to_string()));
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
