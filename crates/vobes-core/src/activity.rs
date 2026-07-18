//! Activity tracking models.

use chrono::{DateTime, Utc};

use crate::error::VobeId;

/// Kind of activity recorded for a vobe.
///
/// Append-only design. New kinds are added without breaking existing
/// records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ActivityKind {
    /// User opened the project.
    Opened,
    /// Filesystem change detected.
    Modified,
    /// Git commit recorded.
    Committed,
    /// Scanner picked up the project.
    Scanned,
    /// First time tracked by Vobes.
    Created,
    /// User explicitly closed (future).
    Closed,
    /// User added/changed tags.
    Tagged,
    /// User edited notes.
    Noted,
}

/// One event in a vobe's lifetime.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityEvent {
    /// Monotonic event id (storage-assigned).
    #[serde(default)]
    pub id: Option<u64>,
    /// Which vobe this event is about.
    pub vobe_id: VobeId,
    /// What kind of event.
    pub kind: ActivityKind,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Free-form context (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ActivityEvent {
    /// Create a new event at the current time.
    pub fn now(vobe_id: VobeId, kind: ActivityKind) -> Self {
        Self {
            id: None,
            vobe_id,
            kind,
            timestamp: Utc::now(),
            detail: None,
        }
    }

    /// Attach a detail string to the event.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}
