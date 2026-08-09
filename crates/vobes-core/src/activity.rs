//! Activity tracking models.

use chrono::{DateTime, Utc};

use crate::error::VobeId;

/// Kind of activity recorded for a vobe.
///
/// Append-only design. New kinds are added without breaking existing
/// records — callers must handle unknown variants gracefully when
/// reading older data.
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

impl ActivityKind {
    /// Short human label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Opened => "opened",
            Self::Modified => "modified",
            Self::Committed => "committed",
            Self::Scanned => "scanned",
            Self::Created => "created",
            Self::Closed => "closed",
            Self::Tagged => "tagged",
            Self::Noted => "noted",
        }
    }
}

impl std::fmt::Display for ActivityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
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
    /// Who triggered the event: `"human"` (default), an agent name
    /// (`"agent"`), or any opaque label set via `VOBES_ACTOR`. Used
    /// by the desktop activity feed to filter "what my agent touched
    /// today". Defaults to `"human"` for events from older
    /// snapshots that predate the field.
    #[serde(default = "default_actor")]
    pub actor: String,
}

/// Default actor label used when an event omits the field.
pub fn default_actor() -> String {
    "human".to_string()
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
            actor: default_actor(),
        }
    }

    /// Create a new event with the actor sourced from
    /// `VOBES_ACTOR`. Use this from CLI/desktop mutation paths so
    /// agents driving `vbs` are attributed without each call site
    /// repeating the env lookup.
    pub fn now_env(vobe_id: VobeId, kind: ActivityKind) -> Self {
        Self::now(vobe_id, kind).with_actor(actor_from_env())
    }

    /// Attach a detail string to the event.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Attach a storage-assigned id.
    pub fn with_id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    /// Override the actor label on this event. Callers building events
    /// from CLI/desktop code should pass `actor_from_env()` here so
    /// the `VOBES_ACTOR` environment variable is honored.
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = actor.into();
        self
    }
}

/// Resolve the actor label for a CLI/desktop invocation.
///
/// Reads the `VOBES_ACTOR` environment variable; falls back to
/// `"human"` when unset (or set to an empty string). Agents driving
/// `vbs` via a shell should export this — e.g. `VOBES_ACTOR=agent vbs open api`.
pub fn actor_from_env() -> String {
    std::env::var("VOBES_ACTOR")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(default_actor)
}
