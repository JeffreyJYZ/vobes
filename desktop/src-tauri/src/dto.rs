//! DTOs (data transfer objects) serialized to the frontend.

use serde::{Deserialize, Serialize};

/// Vobe DTO — a trimmed view of `vobes_core::Vobe` for IPC.
///
/// Field names are serialized as-is (snake_case) to match the frontend
/// TypeScript types exactly (no camelCase rename).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VobeDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub framework: Option<String>,
    pub language: Option<String>,
    pub package_manager: Option<String>,
    pub created_at: String,
    pub last_opened: Option<String>,
    pub last_modified: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub pinned: bool,
    pub git: Option<GitInfoDto>,
}

/// Git info DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfoDto {
    pub branch: String,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    pub last_commit: Option<CommitDto>,
}

/// Commit DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDto {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

/// Activity event DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDto {
    pub id: Option<u64>,
    pub vobe_id: String,
    pub kind: String,
    pub timestamp: String,
    pub detail: Option<String>,
    /// Who triggered the event: `"human"` (default), an agent name
    /// (`"agent"`), or any opaque label set via `VOBES_ACTOR`.
    pub actor: String,
}

impl From<&vobes_core::Vobe> for VobeDto {
    fn from(v: &vobes_core::Vobe) -> Self {
        Self {
            id: v.id.as_str().to_string(),
            name: v.name.clone(),
            path: v.path.to_string_lossy().to_string(),
            framework: v.framework.clone(),
            language: v.language.clone(),
            package_manager: v.package_manager.clone(),
            created_at: v.created_at.to_rfc3339(),
            last_opened: v.last_opened.map(|t| t.to_rfc3339()),
            last_modified: v.last_modified.map(|t| t.to_rfc3339()),
            tags: v.tags.clone(),
            notes: v.notes.clone(),
            pinned: v.pinned,
            git: v.git.as_ref().map(|g| GitInfoDto {
                branch: g.branch.clone(),
                dirty: g.dirty,
                ahead: g.ahead,
                behind: g.behind,
                last_commit: g.last_commit.as_ref().map(|c| CommitDto {
                    hash: c.hash.clone(),
                    message: c.message.clone(),
                    author: c.author.clone(),
                    date: c.date.to_rfc3339(),
                }),
            }),
        }
    }
}

impl From<&vobes_core::ActivityEvent> for ActivityDto {
    fn from(e: &vobes_core::ActivityEvent) -> Self {
        Self {
            id: e.id,
            vobe_id: e.vobe_id.as_str().to_string(),
            kind: e.kind.label().to_string(),
            timestamp: e.timestamp.to_rfc3339(),
            detail: e.detail.clone(),
            actor: e.actor.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vobes_core::{ActivityEvent, ActivityKind, Vobe, VobeId};

    #[test]
    fn snake_case_field_names_match_frontend_types() {
        // The frontend types in desktop/src/lib/types.ts mirror these
        // field names. A silent rename here breaks the UI without a
        // compile error — pin the names so a stray camelCase is loud.
        let vobe = Vobe::new("demo", std::path::Path::new("/tmp/demo"));
        let v_json = serde_json::to_string(&VobeDto::from(&vobe)).unwrap();
        for k in [
            "package_manager",
            "last_modified",
            "last_opened",
            "created_at",
            "git",
            "pinned",
        ] {
            assert!(
                v_json.contains(&format!("\"{k}\"")),
                "VobeDto missing snake_case field {k}: {v_json}"
            );
        }

        let ev = ActivityEvent::now(VobeId::from_string("abc"), ActivityKind::Opened);
        let a_json = serde_json::to_string(&ActivityDto::from(&ev)).unwrap();
        for k in ["vobe_id", "kind", "timestamp", "actor"] {
            assert!(
                a_json.contains(&format!("\"{k}\"")),
                "ActivityDto missing snake_case field {k}: {a_json}"
            );
        }
        // kind is the lowercase human label, not the PascalCase variant.
        assert!(a_json.contains("\"opened\""), "got {a_json}");
    }

    #[test]
    fn activity_kind_label_is_human_readable() {
        // The frontend renders e.kind directly. If the DTO started
        // sending debug-formatted variants, the activity feed would
        // show "Opened" instead of "opened".
        for (kind, label) in [
            (ActivityKind::Opened, "opened"),
            (ActivityKind::Scanned, "scanned"),
            (ActivityKind::Created, "created"),
            (ActivityKind::Tagged, "tagged"),
        ] {
            let ev = ActivityEvent::now(VobeId::from_string("x"), kind);
            let dto = ActivityDto::from(&ev);
            assert_eq!(dto.kind, label);
        }
    }
}
