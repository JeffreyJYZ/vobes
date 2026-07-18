//! Shared error types and aliases.

use std::fmt;
use std::str::FromStr;

/// Stable identifier for a vobe.
///
/// Opaque, generated, never reused. Serialized as a string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct VobeId(pub String);

impl VobeId {
    /// Generate a fresh id from the current monotonic-ish timestamp.
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(format!("vobe_{nanos:x}"))
    }

    /// Wrap an existing id string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for VobeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VobeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for VobeId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for VobeId {
    type Err = ParseIdError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseIdError::Empty);
        }
        Ok(Self(s.to_string()))
    }
}

/// Errors from parsing a [`VobeId`] from a string.
#[derive(thiserror::Error, Debug)]
pub enum ParseIdError {
    /// Input was empty.
    #[error("vobe id cannot be empty")]
    Empty,
}

/// Unified error type for the Vobes ecosystem.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization / deserialization error.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Storage error (e.g. SQLite).
    #[error("storage error: {0}")]
    Storage(String),

    /// Git error.
    #[error("git error: {0}")]
    Git(String),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Scan error.
    #[error("scan error: {0}")]
    Scan(String),

    /// Vobe not found.
    #[error("vobe not found: {0}")]
    NotFound(String),

    /// Invalid id.
    #[error("invalid id: {0}")]
    InvalidId(#[from] ParseIdError),

    /// Generic internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Build a storage error from any string-like source.
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    /// Build a git error from any string-like source.
    pub fn git(msg: impl Into<String>) -> Self {
        Self::Git(msg.into())
    }

    /// Build a config error from any string-like source.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Build a scan error from any string-like source.
    pub fn scan(msg: impl Into<String>) -> Self {
        Self::Scan(msg.into())
    }

    /// Build a not-found error from any string-like source.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Build an internal error from any string-like source.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Alias for fallible operations across the vobes stack.
pub type Result<T> = std::result::Result<T, Error>;

impl serde::Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
