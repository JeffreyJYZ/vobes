//! Vobe model — one software project.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::error::VobeId;
use crate::git::GitInfo;

/// Default tag marking a vobe as archived (ignored by default listings).
pub const ARCHIVED_TAG: &str = "archived";

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

    /// Set the git state.
    pub fn with_git(mut self, git: GitInfo) -> Self {
        self.git = Some(git);
        self
    }

    /// Set the framework.
    pub fn with_framework(mut self, framework: impl Into<String>) -> Self {
        self.framework = Some(framework.into());
        self
    }

    /// Set the language.
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the package manager.
    pub fn with_package_manager(mut self, package_manager: impl Into<String>) -> Self {
        self.package_manager = Some(package_manager.into());
        self
    }

    /// Set the notes content.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Add a tag if not already present. Returns `true` if added.
    pub fn add_tag(&mut self, tag: impl Into<String>) -> bool {
        let tag = tag.into();
        if self.tags.iter().any(|t| t == &tag) {
            false
        } else {
            self.tags.push(tag);
            true
        }
    }

    /// Remove a tag. Returns `true` if it was present.
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let before = self.tags.len();
        self.tags.retain(|t| t != tag);
        before != self.tags.len()
    }

    /// Whether the vobe has a given tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Convenience: is this vobe archived?
    pub fn is_archived(&self) -> bool {
        self.has_tag(ARCHIVED_TAG)
    }

    /// Convenience: does the vobe have uncommitted changes?
    pub fn is_dirty(&self) -> bool {
        self.git.as_ref().is_some_and(|g| g.dirty)
    }

    /// Convenience: does the vobe have commits to push?
    pub fn has_unpushed(&self) -> bool {
        self.git.as_ref().is_some_and(|g| g.ahead > 0)
    }

    /// Convenience: does the vobe have commits to pull?
    pub fn has_unpulled(&self) -> bool {
        self.git.as_ref().is_some_and(|g| g.behind > 0)
    }

    /// Mark the vobe as opened now.
    pub fn touch_opened(&mut self) {
        self.last_opened = Some(Utc::now());
    }

    /// Mark the vobe as modified now.
    pub fn touch_modified(&mut self) {
        self.last_modified = Some(Utc::now());
    }

    /// Store a custom metadata field.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Read a custom metadata field.
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }
}

impl Default for Vobe {
    fn default() -> Self {
        Self::new("", PathBuf::new())
    }
}
