//! Schema definition and migration runner.

use rusqlite::Connection;

use vobes_core::Result;

/// Current schema version. Increment when migrations are added.
pub const SCHEMA_VERSION: u32 = 1;

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

/// Apply migrations to a fresh or existing connection.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_V1)
        .map_err(|e| vobes_core::Error::storage(format!("migrate: {e}")))?;
    conn.execute(
        "INSERT OR IGNORE INTO schema_version (version) VALUES (?1)",
        rusqlite::params![SCHEMA_VERSION],
    )
    .map_err(|e| vobes_core::Error::storage(format!("record version: {e}")))?;
    Ok(())
}
