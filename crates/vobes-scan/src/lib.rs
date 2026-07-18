//! Vobes scanning engine.
//!
//! Walks configured roots and produces vobe candidates via modular
//! detectors. The actual detector implementations arrive in Phase 2;
//! this module defines the trait surface and shared types.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use std::path::{Path, PathBuf};
use vobes_core::Result;

/// Built-in directories always excluded from scan descent.
///
/// User config may add more via `scan.exclude`. These cannot be
/// overridden — they are always excluded to keep scans fast and safe.
pub const BUILTIN_EXCLUDES: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    ".cache",
    "vendor",
    ".next",
    ".venv",
    ".idea",
    ".vscode",
];

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
    fn scan(&self, root: &Path) -> Result<Vec<(PathBuf, Detection)>>;
}

/// Whether a directory name should be excluded from descent.
///
/// Matches against the built-in excludes plus user-supplied extras.
pub fn is_excluded(name: &str, extra: &[String]) -> bool {
    if BUILTIN_EXCLUDES.contains(&name) {
        return true;
    }
    extra.iter().any(|e| e == name)
}
