//! Shared error types and aliases.

use std::fmt;

/// Stable identifier for a vobe.
///
/// Opaque, generated, never reused. Serialized as a string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct VobeId(pub String);

impl VobeId {
    /// Generate a fresh random id.
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Self(format!("vobe_{nanos:x}"))
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

    /// Generic internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Alias for fallible operations across the vobes stack.
pub type Result<T> = std::result::Result<T, Error>;
