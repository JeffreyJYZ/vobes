//! Public store types: filter, sort, export snapshot.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use vobes_core::{ActivityEvent, Vobe};

/// Filter applied when listing vobes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Filter {
    /// Filter by tag (all vobes with this tag).
    pub tag: Option<String>,
    /// Only vobes modified since this time.
    pub modified_since: Option<DateTime<Utc>>,
    /// Only vobes whose `git.dirty` is true.
    pub only_dirty: bool,
    /// Exclude vobes tagged `archived`.
    pub exclude_archived: bool,
}

impl Filter {
    /// Empty filter (matches everything).
    pub fn all() -> Self {
        Self::default()
    }

    /// Only show vobes with this tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Only show vobes with uncommitted changes.
    pub fn only_dirty(mut self) -> Self {
        self.only_dirty = true;
        self
    }

    /// Hide archived vobes from the listing.
    pub fn exclude_archived(mut self) -> Self {
        self.exclude_archived = true;
        self
    }

    /// Only show vobes modified since the given timestamp.
    pub fn modified_since(mut self, since: DateTime<Utc>) -> Self {
        self.modified_since = Some(since);
        self
    }
}

/// Sort order for listing vobes.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    /// By name, ascending.
    Name,
    /// By creation time, newest first.
    CreatedAt,
    /// By last opened, newest first.
    LastOpened,
    /// By last modified, newest first.
    #[default]
    LastModified,
}

/// Shape of a JSON export snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSnapshot {
    /// Export format version.
    pub version: u32,
    /// When the snapshot was produced.
    pub exported_at: DateTime<Utc>,
    /// All vobes at export time.
    pub vobes: Vec<Vobe>,
    /// All recorded activity events.
    pub activity: Vec<ActivityEvent>,
}

impl ExportSnapshot {
    /// Current export format version.
    pub const CURRENT_VERSION: u32 = 1;

    /// Build a snapshot from the current state.
    pub fn new(vobes: Vec<Vobe>, activity: Vec<ActivityEvent>) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            exported_at: Utc::now(),
            vobes,
            activity,
        }
    }
}
