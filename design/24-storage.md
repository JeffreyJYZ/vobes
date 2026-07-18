# Storage

Two storage surfaces, one truth.

## Primary: SQLite

- Embedded, zero-config, fast.
- Lives at `~/.vobes/vobes.db` (platform-resolved).
- Schema migrations versioned.
- One process can hold a write lock; the other platform reads.

### Why SQLite

- Embedded, no server.
- Fast for our scale (hundreds of vobes, thousands of events).
- Queryable.
- Future agents can read it directly.
- Cross-platform.

### Tables

```sql
CREATE TABLE vobes (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    path            TEXT NOT NULL UNIQUE,
    framework       TEXT,
    language        TEXT,
    package_manager TEXT,
    created_at      TEXT NOT NULL,
    last_opened     TEXT,
    last_modified   TEXT,
    tags            TEXT,        -- JSON array
    notes           TEXT,
    metadata        TEXT         -- JSON object
);

CREATE TABLE activity (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    vobe_id     TEXT NOT NULL REFERENCES vobes(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    timestamp   TEXT NOT NULL,
    detail      TEXT
);

CREATE TABLE git_cache (
    vobe_id              TEXT PRIMARY KEY REFERENCES vobes(id) ON DELETE CASCADE,
    branch               TEXT,
    dirty                INTEGER NOT NULL DEFAULT 0,
    ahead                INTEGER NOT NULL DEFAULT 0,
    behind               INTEGER NOT NULL DEFAULT 0,
    last_commit_hash     TEXT,
    last_commit_message  TEXT,
    last_commit_author   TEXT,
    last_commit_date     TEXT,
    refreshed_at         TEXT NOT NULL
);

CREATE INDEX idx_activity_vobe ON activity(vobe_id, timestamp DESC);
CREATE INDEX idx_activity_time ON activity(timestamp DESC);
```

## Export: JSON

- Human-readable snapshot of all vobes + activity.
- Triggered by `vbs export`.
- Written to `~/.vobes/snapshots/vobes-YYYY-MM-DD.json` or custom path.
- Git-friendly, diff-friendly.

### Shape

```json
{
  "version": 1,
  "exported_at": "2026-07-18T10:00:00Z",
  "vobes": [
    {
      "id": "...",
      "name": "Vobes",
      "path": "/Users/jyz/dev/vobes",
      "framework": "Tauri",
      "language": "Rust",
      "package_manager": "cargo",
      "tags": ["work", "personal"],
      "notes": "Active development"
    }
  ],
  "activity": [
    {
      "vobe_id": "...",
      "kind": "Opened",
      "timestamp": "2026-07-18T09:55:00Z",
      "detail": null
    }
  ]
}
```

## Why Both

- **SQLite** is the system of record. Fast queries, concurrent reads.
- **JSON** is for human inspection, backups, migration, git history.

They are not in sync in real time — export is an explicit command.

## Migrations

Stored in `vobes-store/migrations/`. Numbered:

```
migrations/
├── 0001_initial.sql
├── 0002_add_metadata.sql
└── 0003_*.sql
```

Applied in order. Tracked in a `schema_version` table.

## Crate

`vobes-store` exposes a `Store` trait so the platform crates consume
storage through a stable interface:

```rust
trait Store {
    fn upsert_vobe(&self, vobe: &Vobe) -> Result<()>;
    fn get_vobe(&self, id: &VobeId) -> Result<Option<Vobe>>;
    fn list_vobes(&self, filter: &Filter) -> Result<Vec<Vobe>>;
    fn record_activity(&self, event: &ActivityEvent) -> Result<()>;
    fn recent_activity(&self, limit: usize) -> Result<Vec<ActivityEvent>>;
    fn export_json(&self, path: &Path) -> Result<()>;
    fn import_json(&self, path: &Path) -> Result<()>;
}
```
