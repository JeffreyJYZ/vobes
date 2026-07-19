//! SQLite-backed `Store` implementation.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use vobes_core::{normalize, ActivityEvent, ActivityKind, Commit, GitInfo, Result, Vobe, VobeId};

use crate::model::{Filter, Sort};
use crate::schema::migrate;
use crate::Store;

/// A SQLite-backed store. Single connection, guarded by a mutex.
pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    /// Open (or create) a store at `path`. Applies migrations.
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| vobes_core::Error::storage(format!("create dir: {e}")))?;
        }
        let conn =
            Connection::open(path).map_err(|e| vobes_core::Error::storage(format!("open: {e}")))?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|e| vobes_core::Error::storage(format!("pragma: {e}")))?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open an in-memory store (useful for tests).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| vobes_core::Error::storage(format!("open in memory: {e}")))?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| vobes_core::Error::internal(format!("store lock: {e}")))?;
        f(&conn)
    }
}

impl Store for SqliteStore {
    fn upsert_vobe(&self, vobe: &Vobe) -> Result<()> {
        self.with_conn(|conn| {
            let tags = serde_json::to_string(&vobe.tags)
                .map_err(|e| vobes_core::Error::storage(format!("encode tags: {e}")))?;
            let metadata = serde_json::to_string(&vobe.metadata)
                .map_err(|e| vobes_core::Error::storage(format!("encode metadata: {e}")))?;

            let (
                git_branch,
                git_dirty,
                git_ahead,
                git_behind,
                git_last_hash,
                git_last_msg,
                git_last_author,
                git_last_date,
            ) = match &vobe.git {
                Some(g) => (
                    Some(g.branch.as_str()),
                    g.dirty as i64,
                    g.ahead as i64,
                    g.behind as i64,
                    g.last_commit.as_ref().map(|c| c.hash.as_str()),
                    g.last_commit.as_ref().map(|c| c.message.as_str()),
                    g.last_commit.as_ref().map(|c| c.author.as_str()),
                    g.last_commit.as_ref().map(|c| c.date.to_rfc3339()),
                ),
                None => (None, 0, 0, 0, None, None, None, None),
            };

            conn.execute(
                "INSERT INTO vobes (
                    id, name, path, framework, language, package_manager,
                    created_at, last_opened, last_modified, tags, notes, metadata,
                    git_branch, git_dirty, git_ahead, git_behind,
                    git_last_hash, git_last_msg, git_last_author, git_last_date,
                    refreshed_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6,
                    ?7, ?8, ?9, ?10, ?11, ?12,
                    ?13, ?14, ?15, ?16,
                    ?17, ?18, ?19, ?20,
                    ?21
                )
                ON CONFLICT(id) DO UPDATE SET
                    name=excluded.name,
                    path=excluded.path,
                    framework=excluded.framework,
                    language=excluded.language,
                    package_manager=excluded.package_manager,
                    last_opened=excluded.last_opened,
                    last_modified=excluded.last_modified,
                    tags=excluded.tags,
                    notes=excluded.notes,
                    metadata=excluded.metadata,
                    git_branch=excluded.git_branch,
                    git_dirty=excluded.git_dirty,
                    git_ahead=excluded.git_ahead,
                    git_behind=excluded.git_behind,
                    git_last_hash=excluded.git_last_hash,
                    git_last_msg=excluded.git_last_msg,
                    git_last_author=excluded.git_last_author,
                    git_last_date=excluded.git_last_date,
                    refreshed_at=excluded.refreshed_at",
                params![
                    vobe.id.as_str(),
                    vobe.name,
                    vobes_core::normalize(&vobe.path).to_string_lossy(),
                    vobe.framework,
                    vobe.language,
                    vobe.package_manager,
                    vobe.created_at.to_rfc3339(),
                    vobe.last_opened.map(|t| t.to_rfc3339()),
                    vobe.last_modified.map(|t| t.to_rfc3339()),
                    tags,
                    vobe.notes,
                    metadata,
                    git_branch,
                    git_dirty,
                    git_ahead,
                    git_behind,
                    git_last_hash,
                    git_last_msg,
                    git_last_author,
                    git_last_date,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|e| vobes_core::Error::storage(format!("upsert vobe: {e}")))?;
            Ok(())
        })
    }

    fn get_vobe(&self, id: &VobeId) -> Result<Option<Vobe>> {
        self.with_conn(|conn| fetch_vobe(conn, "id = ?1", params![id.as_str()]))
    }

    fn get_vobe_by_name(&self, name: &str) -> Result<Option<Vobe>> {
        self.with_conn(|conn| fetch_vobe(conn, "name = ?1", params![name]))
    }

    fn get_vobe_by_path(&self, path: &Path) -> Result<Option<Vobe>> {
        let normalized = normalize(path);
        let s = normalized.to_string_lossy();
        self.with_conn(|conn| fetch_vobe(conn, "path = ?1", params![s.as_ref()]))
    }

    fn list_vobes(&self, filter: &Filter, sort: Sort) -> Result<Vec<Vobe>> {
        self.with_conn(|conn| {
            let mut sql = String::from("SELECT * FROM vobes WHERE 1=1");
            let mut args: Vec<rusqlite::types::Value> = Vec::new();
            let mut next_idx = 1usize;

            if let Some(tag) = &filter.tag {
                sql.push_str(&format!(" AND tags LIKE ?{next_idx}"));
                args.push(format!("%\"{tag}\"%").into());
                next_idx += 1;
            }
            if let Some(since) = filter.modified_since {
                sql.push_str(&format!(
                    " AND last_modified IS NOT NULL AND last_modified >= ?{next_idx}"
                ));
                args.push(since.to_rfc3339().into());
                next_idx += 1;
            }
            let _ = next_idx;
            if filter.only_dirty {
                sql.push_str(" AND git_dirty = 1");
            }
            if filter.exclude_archived {
                sql.push_str(" AND tags NOT LIKE '%\"archived\"%'");
            }

            match sort {
                Sort::Name => sql.push_str(" ORDER BY name ASC"),
                Sort::CreatedAt => sql.push_str(" ORDER BY created_at DESC"),
                Sort::LastOpened => sql.push_str(" ORDER BY last_opened DESC NULLS LAST"),
                Sort::LastModified => sql.push_str(" ORDER BY last_modified DESC NULLS LAST"),
            }

            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| vobes_core::Error::storage(format!("list prepare: {e}")))?;
            let rows = stmt
                .query_map(rusqlite::params_from_iter(args.iter()), row_to_vobe)
                .map_err(|e| vobes_core::Error::storage(format!("list query: {e}")))?;
            let mut vobes = Vec::new();
            for r in rows {
                vobes.push(r.map_err(|e| vobes_core::Error::storage(format!("row: {e}")))?);
            }
            Ok(vobes)
        })
    }

    fn delete_vobe(&self, id: &VobeId) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM vobes WHERE id = ?1", params![id.as_str()])
                .map_err(|e| vobes_core::Error::storage(format!("delete vobe: {e}")))?;
            Ok(())
        })
    }

    fn purge_all(&self) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute("DELETE FROM activity", [])
                .map_err(|e| vobes_core::Error::storage(format!("purge activity: {e}")))?;
            conn.execute("DELETE FROM vobes", [])
                .map_err(|e| vobes_core::Error::storage(format!("purge vobes: {e}")))?;
            Ok(())
        })
    }

    fn record_activity(&self, event: &ActivityEvent) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO activity (vobe_id, kind, timestamp, detail) VALUES (?1, ?2, ?3, ?4)",
                params![
                    event.vobe_id.as_str(),
                    kind_to_str(event.kind),
                    event.timestamp.to_rfc3339(),
                    event.detail,
                ],
            )
            .map_err(|e| vobes_core::Error::storage(format!("insert activity: {e}")))?;
            Ok(())
        })
    }

    fn recent_activity(&self, limit: usize) -> Result<Vec<ActivityEvent>> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, vobe_id, kind, timestamp, detail FROM activity
                     ORDER BY timestamp DESC, id DESC LIMIT ?1",
                )
                .map_err(|e| vobes_core::Error::storage(format!("recent prepare: {e}")))?;
            let rows = stmt
                .query_map(params![limit as i64], row_to_activity)
                .map_err(|e| vobes_core::Error::storage(format!("recent query: {e}")))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| vobes_core::Error::storage(format!("row: {e}")))?);
            }
            Ok(out)
        })
    }

    fn vobe_activity(&self, vobe_id: &VobeId, limit: usize) -> Result<Vec<ActivityEvent>> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, vobe_id, kind, timestamp, detail FROM activity
                     WHERE vobe_id = ?1
                     ORDER BY timestamp DESC, id DESC LIMIT ?2",
                )
                .map_err(|e| vobes_core::Error::storage(format!("vobe_activity prepare: {e}")))?;
            let rows = stmt
                .query_map(params![vobe_id.as_str(), limit as i64], row_to_activity)
                .map_err(|e| vobes_core::Error::storage(format!("vobe_activity query: {e}")))?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r.map_err(|e| vobes_core::Error::storage(format!("row: {e}")))?);
            }
            Ok(out)
        })
    }

    fn export_json(&self, path: &Path) -> Result<()> {
        let vobes = self.list_vobes(&Filter::all(), Sort::Name)?;
        let activity = self.recent_activity(usize::MAX)?;
        crate::json::export_to_file(path, &vobes, &activity)
    }

    fn import_json(&self, path: &Path) -> Result<()> {
        let snap = crate::json::import_from_file(path)?;
        self.with_conn(|conn| {
            // Best-effort import — clean slate within a transaction.
            conn.execute_batch("BEGIN;")
                .map_err(|e| vobes_core::Error::storage(format!("begin: {e}")))?;
            for v in &snap.vobes {
                upsert_vobe_inline(conn, v)?;
            }
            for e in &snap.activity {
                conn.execute(
                    "INSERT INTO activity (vobe_id, kind, timestamp, detail) VALUES (?1, ?2, ?3, ?4)",
                    params![
                        e.vobe_id.as_str(),
                        kind_to_str(e.kind),
                        e.timestamp.to_rfc3339(),
                        e.detail
                    ],
                )
                .map_err(|er| vobes_core::Error::storage(format!("import activity: {er}")))?;
            }
            conn.execute_batch("COMMIT;")
                .map_err(|e| vobes_core::Error::storage(format!("commit: {e}")))?;
            Ok(())
        })
    }
}

fn upsert_vobe_inline(conn: &Connection, vobe: &Vobe) -> Result<()> {
    let tags = serde_json::to_string(&vobe.tags)
        .map_err(|e| vobes_core::Error::storage(format!("encode tags: {e}")))?;
    let metadata = serde_json::to_string(&vobe.metadata)
        .map_err(|e| vobes_core::Error::storage(format!("encode metadata: {e}")))?;
    let (
        git_branch,
        git_dirty,
        git_ahead,
        git_behind,
        git_last_hash,
        git_last_msg,
        git_last_author,
        git_last_date,
    ) = match &vobe.git {
        Some(g) => (
            Some(g.branch.as_str()),
            g.dirty as i64,
            g.ahead as i64,
            g.behind as i64,
            g.last_commit.as_ref().map(|c| c.hash.as_str()),
            g.last_commit.as_ref().map(|c| c.message.as_str()),
            g.last_commit.as_ref().map(|c| c.author.as_str()),
            g.last_commit.as_ref().map(|c| c.date.to_rfc3339()),
        ),
        None => (None, 0, 0, 0, None, None, None, None),
    };

    conn.execute(
        "INSERT INTO vobes (
            id, name, path, framework, language, package_manager,
            created_at, last_opened, last_modified, tags, notes, metadata,
            git_branch, git_dirty, git_ahead, git_behind,
            git_last_hash, git_last_msg, git_last_author, git_last_date,
            refreshed_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            ?7, ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16,
            ?17, ?18, ?19, ?20,
            ?21
        )
        ON CONFLICT(id) DO UPDATE SET
            name=excluded.name, path=excluded.path, framework=excluded.framework,
            language=excluded.language, package_manager=excluded.package_manager,
            last_opened=excluded.last_opened, last_modified=excluded.last_modified,
            tags=excluded.tags, notes=excluded.notes, metadata=excluded.metadata,
            git_branch=excluded.git_branch, git_dirty=excluded.git_dirty,
            git_ahead=excluded.git_ahead, git_behind=excluded.git_behind,
            git_last_hash=excluded.git_last_hash, git_last_msg=excluded.git_last_msg,
            git_last_author=excluded.git_last_author, git_last_date=excluded.git_last_date,
            refreshed_at=excluded.refreshed_at",
        params![
            vobe.id.as_str(),
            vobe.name,
            vobes_core::normalize(&vobe.path).to_string_lossy(),
            vobe.framework,
            vobe.language,
            vobe.package_manager,
            vobe.created_at.to_rfc3339(),
            vobe.last_opened.map(|t| t.to_rfc3339()),
            vobe.last_modified.map(|t| t.to_rfc3339()),
            tags,
            vobe.notes,
            metadata,
            git_branch,
            git_dirty,
            git_ahead,
            git_behind,
            git_last_hash,
            git_last_msg,
            git_last_author,
            git_last_date,
            Utc::now().to_rfc3339(),
        ],
    )
    .map_err(|e| vobes_core::Error::storage(format!("import upsert: {e}")))?;
    Ok(())
}

fn fetch_vobe(
    conn: &Connection,
    where_clause: &str,
    args: impl rusqlite::Params,
) -> Result<Option<Vobe>> {
    let sql = format!("SELECT * FROM vobes WHERE {where_clause}");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| vobes_core::Error::storage(format!("prepare: {e}")))?;
    let vobe = stmt
        .query_row(args, row_to_vobe)
        .optional()
        .map_err(|e| vobes_core::Error::storage(format!("fetch: {e}")))?;
    Ok(vobe)
}

fn row_to_vobe(row: &rusqlite::Row<'_>) -> rusqlite::Result<Vobe> {
    let id: String = row.get("id")?;
    let name: String = row.get("name")?;
    let path: String = row.get("path")?;
    let framework: Option<String> = row.get("framework")?;
    let language: Option<String> = row.get("language")?;
    let package_manager: Option<String> = row.get("package_manager")?;
    let created_at: String = row.get("created_at")?;
    let last_opened: Option<String> = row.get("last_opened")?;
    let last_modified: Option<String> = row.get("last_modified")?;
    let tags_json: String = row.get("tags")?;
    let notes: Option<String> = row.get("notes")?;
    let metadata_json: String = row.get("metadata")?;
    let git_branch: Option<String> = row.get("git_branch")?;
    let git_dirty: i64 = row.get("git_dirty")?;
    let git_ahead: i64 = row.get("git_ahead")?;
    let git_behind: i64 = row.get("git_behind")?;
    let git_last_hash: Option<String> = row.get("git_last_hash")?;
    let git_last_msg: Option<String> = row.get("git_last_msg")?;
    let git_last_author: Option<String> = row.get("git_last_author")?;
    let git_last_date: Option<String> = row.get("git_last_date")?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let metadata: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&metadata_json).unwrap_or_default();

    let last_commit = match (git_last_hash, git_last_msg, git_last_author, git_last_date) {
        (Some(hash), Some(message), Some(author), Some(date)) => {
            let date = DateTime::parse_from_rfc3339(&date)
                .ok()
                .map(|d| d.with_timezone(&Utc));
            Some(Commit {
                hash,
                message,
                author,
                date: date.unwrap_or_else(Utc::now),
            })
        }
        _ => None,
    };

    let git = git_branch.map(|branch| GitInfo {
        branch,
        dirty: git_dirty != 0,
        ahead: git_ahead.max(0) as u32,
        behind: git_behind.max(0) as u32,
        last_commit,
    });

    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let last_opened = last_opened
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&Utc));
    let last_modified = last_modified
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&Utc));

    Ok(Vobe {
        id: VobeId::from_string(id),
        name,
        path: normalize(&PathBuf::from(path)),
        git,
        framework,
        language,
        package_manager,
        created_at,
        last_opened,
        last_modified,
        tags,
        notes,
        metadata,
    })
}

fn row_to_activity(row: &rusqlite::Row<'_>) -> rusqlite::Result<ActivityEvent> {
    let id: i64 = row.get("id")?;
    let vobe_id: String = row.get("vobe_id")?;
    let kind: String = row.get("kind")?;
    let timestamp: String = row.get("timestamp")?;
    let detail: Option<String> = row.get("detail")?;

    let kind = kind_from_str(&kind);
    let timestamp = DateTime::parse_from_rfc3339(&timestamp)
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    Ok(ActivityEvent {
        id: Some(id as u64),
        vobe_id: VobeId::from_string(vobe_id),
        kind,
        timestamp,
        detail,
    })
}

fn kind_to_str(k: ActivityKind) -> &'static str {
    match k {
        ActivityKind::Opened => "Opened",
        ActivityKind::Modified => "Modified",
        ActivityKind::Committed => "Committed",
        ActivityKind::Scanned => "Scanned",
        ActivityKind::Created => "Created",
        ActivityKind::Closed => "Closed",
        ActivityKind::Tagged => "Tagged",
        ActivityKind::Noted => "Noted",
    }
}

fn kind_from_str(s: &str) -> ActivityKind {
    match s {
        "Opened" => ActivityKind::Opened,
        "Modified" => ActivityKind::Modified,
        "Committed" => ActivityKind::Committed,
        "Scanned" => ActivityKind::Scanned,
        "Created" => ActivityKind::Created,
        "Closed" => ActivityKind::Closed,
        "Tagged" => ActivityKind::Tagged,
        "Noted" => ActivityKind::Noted,
        _ => ActivityKind::Scanned,
    }
}
