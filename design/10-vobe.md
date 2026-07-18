# Vobe (Data Model)

A **vobe** is one software project managed by Vobes.

## Definition

```rust
struct Vobe {
    id: VobeId,                       // stable internal id
    name: String,                     // human label
    path: PathBuf,                    // absolute path on disk
    git: Option<GitInfo>,             // git state
    framework: Option<String>,        // e.g. "Next.js", "Axum"
    language: Option<String>,         // e.g. "TypeScript", "Rust"
    package_manager: Option<String>,  // e.g. "pnpm", "cargo"
    created_at: DateTime<Utc>,
    last_opened: Option<DateTime<Utc>>,
    last_modified: Option<DateTime<Utc>>,
    tags: Vec<String>,
    notes: Option<String>,
    metadata: HashMap<String, Value>, // custom, extensible
}
```

## Fields

### Identity

- **id** — opaque, generated, stable across renames/moves.
- **name** — human label. Can be edited.
- **path** — absolute path. Used for scanning and editor launch.

### Source State

- **git** — see [Git](./12-git.md). `None` for non-git projects.
- **framework** — primary framework. Detected, can be overridden.
- **language** — primary language. Detected.
- **package_manager** — primary package manager. Detected.

### Time

- **created_at** — when Vobes first saw this project.
- **last_opened** — last `vbs open` / desktop launch.
- **last_modified** — last filesystem mtime under the project.

### User Data

- **tags** — free-form labels (`work`, `personal`, `archived`…).
- **notes** — markdown allowed, free-form.
- **metadata** — arbitrary `serde_json::Value` for custom fields.

## Extensibility

The `metadata` field is the escape hatch. Vobes itself treats it as
opaque; users or future agents can store anything there.

Tags are intentionally simple. No taxonomy, no hierarchy.

## Lifecycle

1. **Discovered** — scanner finds a directory with a repo/manifest.
2. **Tracked** — stored in the database. Default state.
3. **Opened** — `last_opened` updates. Activity event recorded.
4. **Modified** — filesystem change detected. `last_modified` updates.
5. **Removed** — user marks for removal. Stays in DB, hidden by default.

## Invariants

- `path` is always absolute.
- `id` is unique and never reused.
- `created_at` is set once and never changes.
- `git`, `framework`, `language`, `package_manager` may be `None`.
