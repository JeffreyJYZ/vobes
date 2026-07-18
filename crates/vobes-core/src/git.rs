//! Git information surfaced by Vobes.
//!
//! Read-only. Vobes does not commit, push, branch, or merge.

use chrono::{DateTime, Utc};

/// Surfaced Git state for a vobe.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GitInfo {
    /// Current branch name.
    pub branch: String,
    /// Any unstaged or uncommitted changes?
    pub dirty: bool,
    /// Commits ahead of upstream.
    pub ahead: u32,
    /// Commits behind upstream.
    pub behind: u32,
    /// Most recent commit on the current branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit: Option<Commit>,
}

/// Minimal commit info — enough to display in a dashboard.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Commit {
    /// Commit SHA (full or abbreviated by display layer).
    pub hash: String,
    /// First line of the commit message.
    pub message: String,
    /// Author name (and email if available).
    pub author: String,
    /// Commit timestamp.
    pub date: DateTime<Utc>,
}
