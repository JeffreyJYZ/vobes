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

    /// Hide archived vobes from the listing.
    pub fn exclude_archived(mut self) -> Self {
        self.exclude_archived = true;
        self
    }
}

/// Sort order for listing vobes.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    /// By name, ascending.
    Name,
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
    /// Saved searches, so a view pinned on one machine exists on the
    /// next. Empty for snapshots written before this field existed.
    #[serde(default)]
    pub saved_filters: Vec<SavedFilter>,
}

impl ExportSnapshot {
    /// Current export format version.
    pub const CURRENT_VERSION: u32 = 2;

    /// Build a snapshot from the current state.
    pub fn new(
        vobes: Vec<Vobe>,
        activity: Vec<ActivityEvent>,
        saved_filters: Vec<SavedFilter>,
    ) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            exported_at: Utc::now(),
            vobes,
            activity,
            saved_filters,
        }
    }
}

/// A pinned search query saved by the user. Persistent counterpart
/// of the in-frontend `SavedFilter` type — kept in SQLite so it
/// syncs across machines via JSON export/import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilter {
    /// Stable opaque id (caller-assigned; frontend uses timestamp+rand).
    pub id: String,
    /// Human label shown in the sidebar.
    pub label: String,
    /// Search query string replayed when the filter is activated.
    pub query: String,
    /// When the filter was saved (RFC3339).
    pub created_at: DateTime<Utc>,
}
