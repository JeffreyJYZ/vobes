//! Scanner traits and shared types.

pub mod framework;
pub mod language;
pub mod package;
pub mod repo;

use std::path::Path;
use vobes_core::Result;

/// A scan result for a single path.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Detection {
    /// Whether the path is a git repository.
    pub is_repo: bool,
    /// Detected framework, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    /// Detected primary language, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Detected package manager, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
}

impl Detection {
    /// Empty detection (nothing found).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Whether the detection identified anything at all.
    pub fn is_empty(&self) -> bool {
        !self.is_repo
            && self.framework.is_none()
            && self.language.is_none()
            && self.package_manager.is_none()
    }

    /// Merge another detection into this one — non-empty values win.
    pub fn merge(&mut self, other: Detection) {
        self.is_repo |= other.is_repo;
        if self.framework.is_none() {
            self.framework = other.framework;
        }
        if self.language.is_none() {
            self.language = other.language;
        }
        if self.package_manager.is_none() {
            self.package_manager = other.package_manager;
        }
    }
}

/// Trait implemented by every detector.
///
/// Adding a new framework or language means adding one detector — no
/// core change.
pub trait Detector: Send + Sync {
    /// Detector name (for logging/debugging).
    fn name(&self) -> &str;
    /// Inspect a path and report a detection (or `None`).
    fn detect(&self, path: &Path) -> Result<Option<Detection>>;
}

/// Trait implemented by scanners.
pub trait Scanner: Send + Sync {
    /// Scan a single root directory, returning `(path, detection)` pairs
    /// for every candidate vobe found.
    fn scan(&self, root: &Path) -> Result<Vec<(std::path::PathBuf, Detection)>>;
}
