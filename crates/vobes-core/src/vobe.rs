//! Vobe model — one software project.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::error::VobeId;
use crate::git::GitInfo;

/// One software project managed by Vobes.
///
/// Extensible via `metadata`: Vobes treats it as opaque, users and
/// future agents can store anything there.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vobe {
    /// Stable internal id (never reused, survives renames).
    pub id: VobeId,
    /// Human label. Editable.
    pub name: String,
    /// Absolute path on disk.
    pub path: PathBuf,
    /// Git state, if the project is a repo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
    /// Primary framework (e.g. "Next.js", "Axum").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    /// Primary language (e.g. "TypeScript", "Rust").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Primary package manager (e.g. "pnpm", "cargo").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
    /// When Vobes first saw this project. Never changes.
    pub created_at: DateTime<Utc>,
    /// Last time the user opened this vobe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_opened: Option<DateTime<Utc>>,
    /// Last time the filesystem under the project changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<DateTime<Utc>>,
    /// Free-form tags (e.g. ["work", "personal", "archived"]).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Free-form notes (markdown allowed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Custom, extensible fields. Opaque to Vobes.
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

impl Vobe {
    /// Create a new vobe with the given name and path, generating an id
    /// and setting `created_at` to now.
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: VobeId::new(),
            name: name.into(),
            path: path.into(),
            git: None,
            framework: None,
            language: None,
            package_manager: None,
            created_at: Utc::now(),
            last_opened: None,
            last_modified: None,
            tags: Vec::new(),
            notes: None,
            metadata: HashMap::new(),
        }
    }
}
