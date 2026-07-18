//! Vobes storage crate — SQLite primary, JSON export.
//!
//! The `Store` trait is the stable interface consumed by platform crates.
//! SQLite and JSON implementations arrive in Phase 5.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use std::path::Path;
use vobes_core::{ActivityEvent, Result, Vobe, VobeId};

/// Filter applied when listing vobes.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Filter {
    /// Filter by tag (all vobes with this tag).
    pub tag: Option<String>,
    /// Only vobes modified since this time.
    pub modified_since: Option<chrono::DateTime<chrono::Utc>>,
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
    pub fn modified_since(mut self, since: chrono::DateTime<chrono::Utc>) -> Self {
        self.modified_since = Some(since);
        self
    }
}

/// Shape of a JSON export snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportSnapshot {
    /// Export format version.
    pub version: u32,
    /// When the snapshot was produced.
    pub exported_at: chrono::DateTime<chrono::Utc>,
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
            exported_at: chrono::Utc::now(),
            vobes,
            activity,
        }
    }
}

/// Sort order for listing vobes.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
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

/// Store trait — the stable interface platform crates consume.
pub trait Store: Send + Sync {
    /// Insert or update a vobe.
    fn upsert_vobe(&self, vobe: &Vobe) -> Result<()>;
    /// Fetch a single vobe by id.
    fn get_vobe(&self, id: &VobeId) -> Result<Option<Vobe>>;
    /// Fetch a single vobe by name.
    fn get_vobe_by_name(&self, name: &str) -> Result<Option<Vobe>>;
    /// Fetch a single vobe by path.
    fn get_vobe_by_path(&self, path: &Path) -> Result<Option<Vobe>>;
    /// List vobes matching the filter, sorted.
    fn list_vobes(&self, filter: &Filter, sort: Sort) -> Result<Vec<Vobe>>;
    /// Delete a vobe (cascades to activity).
    fn delete_vobe(&self, id: &VobeId) -> Result<()>;
    /// Record an activity event.
    fn record_activity(&self, event: &ActivityEvent) -> Result<()>;
    /// Most recent N events globally.
    fn recent_activity(&self, limit: usize) -> Result<Vec<ActivityEvent>>;
    /// Most recent N events for a vobe.
    fn vobe_activity(&self, vobe_id: &VobeId, limit: usize) -> Result<Vec<ActivityEvent>>;
    /// Export all data as JSON to the given path.
    fn export_json(&self, path: &Path) -> Result<()>;
    /// Import data from a previous JSON export.
    fn import_json(&self, path: &Path) -> Result<()>;
}
