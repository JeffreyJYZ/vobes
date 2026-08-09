//! Schema definition and migration runner.

use rusqlite::Connection;

use vobes_core::Result;

/// Current schema version. Increment when migrations are added.
pub const SCHEMA_VERSION: u32 = 4;

/// Initial schema. Creates all tables and indexes for v1.
pub const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS vobes (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    path            TEXT NOT NULL UNIQUE,
    framework       TEXT,
    language        TEXT,
    package_manager TEXT,
    created_at      TEXT NOT NULL,
    last_opened     TEXT,
    last_modified   TEXT,
    tags            TEXT NOT NULL DEFAULT '[]',
    notes           TEXT,
    metadata        TEXT NOT NULL DEFAULT '{}',
    git_branch      TEXT,
    git_dirty       INTEGER NOT NULL DEFAULT 0,
    git_ahead       INTEGER NOT NULL DEFAULT 0,
    git_behind      INTEGER NOT NULL DEFAULT 0,
    git_last_hash   TEXT,
    git_last_msg    TEXT,
    git_last_author TEXT,
    git_last_date   TEXT,
    refreshed_at    TEXT
);

CREATE TABLE IF NOT EXISTS activity (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    vobe_id     TEXT NOT NULL REFERENCES vobes(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    timestamp   TEXT NOT NULL,
    detail      TEXT
);

CREATE INDEX IF NOT EXISTS idx_activity_vobe ON activity(vobe_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_activity_time ON activity(timestamp DESC);
"#;

/// Migration to v2: add `pinned` column to vobes.
pub const MIGRATION_V1_TO_V2: &str = r#"
ALTER TABLE vobes ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_vobes_pinned ON vobes(pinned DESC, last_modified DESC);
"#;

/// Migration to v3: add `actor` column to activity. Older rows
/// backfill to `"human"` — matches the `ActivityEvent::default`
/// semantics used by the core model.
pub const MIGRATION_V2_TO_V3: &str = r#"
ALTER TABLE activity ADD COLUMN actor TEXT NOT NULL DEFAULT 'human';
CREATE INDEX IF NOT EXISTS idx_activity_actor ON activity(actor, timestamp DESC);
"#;

/// Migration to v4: create `saved_filters` table so pinned searches
/// persist with the SQLite store and sync across machines via the
/// same export/import snapshot that carries vobes + activity.
pub const MIGRATION_V3_TO_V4: &str = r#"
CREATE TABLE IF NOT EXISTS saved_filters (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    query       TEXT NOT NULL,
    created_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_saved_filters_created ON saved_filters(created_at DESC);
"#;

/// Apply migrations to a fresh or existing connection.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_V1)
        .map_err(|e| vobes_core::Error::storage(format!("migrate: {e}")))?;
    let current: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if current < 2 {
        conn.execute_batch(MIGRATION_V1_TO_V2)
            .map_err(|e| vobes_core::Error::storage(format!("migrate v2: {e}")))?;
    }
    if current < 3 {
        conn.execute_batch(MIGRATION_V2_TO_V3)
            .map_err(|e| vobes_core::Error::storage(format!("migrate v3: {e}")))?;
    }
    if current < 4 {
        conn.execute_batch(MIGRATION_V3_TO_V4)
            .map_err(|e| vobes_core::Error::storage(format!("migrate v4: {e}")))?;
    }
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
        rusqlite::params![SCHEMA_VERSION],
    )
    .map_err(|e| vobes_core::Error::storage(format!("record version: {e}")))?;
    Ok(())
}
