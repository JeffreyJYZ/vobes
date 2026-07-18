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

impl GitInfo {
    /// Build a clean snapshot of a branch.
    pub fn clean(branch: impl Into<String>) -> Self {
        Self {
            branch: branch.into(),
            dirty: false,
            ahead: 0,
            behind: 0,
            last_commit: None,
        }
    }

    /// Branch has uncommitted or untracked changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Branch has commits to push upstream.
    pub fn needs_push(&self) -> bool {
        self.ahead > 0
    }

    /// Branch has commits to pull from upstream.
    pub fn needs_pull(&self) -> bool {
        self.behind > 0
    }

    /// Branch is in sync and clean.
    pub fn is_clean(&self) -> bool {
        !self.dirty && self.ahead == 0 && self.behind == 0
    }
}

/// Minimal commit info — enough to display in a dashboard.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Commit {
    /// Full commit SHA. Display layer may abbreviate.
    pub hash: String,
    /// First line of the commit message.
    pub message: String,
    /// Author name (and email if available).
    pub author: String,
    /// Commit timestamp.
    pub date: DateTime<Utc>,
}

impl Commit {
    /// Abbreviated SHA — first 7 chars.
    pub fn short_hash(&self) -> &str {
        let len = self.hash.len().min(7);
        &self.hash[..len]
    }

    /// First line of the message, truncated to `max` chars.
    pub fn short_message(&self, max: usize) -> String {
        if self.message.len() <= max {
            self.message.clone()
        } else {
            format!("{}…", &self.message[..max])
        }
    }
}
