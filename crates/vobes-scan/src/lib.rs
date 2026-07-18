//! Vobes scanning engine.
//!
//! Walks configured roots and produces vobe candidates via modular
//! detectors. Implementation arrives in Phase 2.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use std::path::{Path, PathBuf};
use vobes_core::Result;

/// A scan result for a single path.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Detection {
    /// Whether the path is a git repository.
    pub is_repo: bool,
    /// Detected framework, if any.
    pub framework: Option<String>,
    /// Detected primary language, if any.
    pub language: Option<String>,
    /// Detected package manager, if any.
    pub package_manager: Option<String>,
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
    /// Scan a single root directory.
    fn scan(&self, root: &Path) -> Result<Vec<(PathBuf, Detection)>>;
}
