//! DTOs (data transfer objects) serialized to the frontend.

use serde::{Deserialize, Serialize};

/// Vobe DTO — a trimmed view of `vobes_core::Vobe` for IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub git: Option<GitInfoDto>,
}

/// Git info DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfoDto {
    pub branch: String,
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    pub last_commit: Option<CommitDto>,
}

/// Commit DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDto {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

/// Activity event DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDto {
    pub id: Option<u64>,
    pub vobe_id: String,
    pub kind: String,
    pub timestamp: String,
    pub detail: Option<String>,
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
            kind: format!("{:?}", e.kind),
            timestamp: e.timestamp.to_rfc3339(),
            detail: e.detail.clone(),
        }
    }
}
