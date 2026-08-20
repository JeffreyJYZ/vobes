//! Vobe model — one software project.

use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::error::VobeId;
use crate::git::GitInfo;

/// Default tag marking a vobe as archived (ignored by default listings).
pub const ARCHIVED_TAG: &str = "archived";

/// One software project managed by Vobes.
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
    /// Whether the user pinned this vobe (always shown first).
    #[serde(default)]
    pub pinned: bool,
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
            pinned: false,
        }
    }

    /// Set the git state.
    pub fn with_git(mut self, git: GitInfo) -> Self {
        self.git = Some(git);
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
}

impl Default for Vobe {
    fn default() -> Self {
        Self::new("", PathBuf::new())
    }
}
