# Dependencies

Rust crate map.

## Workspace-Level (`[workspace.dependencies]`)

### Serialization

| Crate | Purpose |
|---|---|
| `serde` | derive Serialize/Deserialize |
| `serde_json` | JSON for export + custom metadata |
| `toml` | config parsing |

### Errors

| Crate | Purpose |
|---|---|
| `thiserror` | typed error enums |
| `miette` | pretty error reports (optional) |

### CLI

| Crate | Purpose |
|---|---|
| `clap` | argument parsing (derive) |
| `console` | terminal styling |
| `comfy-table` | table output |
| `indicatif` | progress bars |

### Filesystem & Paths

| Crate | Purpose |
|---|---|
| `walkdir` | directory walking |
| `dirs` | platform path resolution |
| `glob` | pattern matching |

### Concurrency

| Crate | Purpose |
|---|---|
| `rayon` | data-parallel work |
| `crossbeam-channel` | event channels (future) |

### Time

| Crate | Purpose |
|---|---|
| `chrono` | DateTime<Utc>, formatting |

### Git

| Crate | Purpose |
|---|---|
| `git2` | libgit2 bindings (cross-platform) |

### Storage

| Crate | Purpose |
|---|---|
| `rusqlite` (bundled) | SQLite (statically linked, no system dep) |

### Logging

| Crate | Purpose |
|---|---|
| `tracing` | structured logs |
| `tracing-subscriber` | subscriber setup |

## Per-Crate Additions

| Crate | Adds |
|---|---|
| `vobes-core` | (workspace only) |
| `vobes-scan` | `regex`, `once_cell` (or `std::sync::LazyLock` on newer Rust) |
| `vobes-git` | `git2` |
| `vobes-activity` | (workspace only) |
| `vobes-store` | `rusqlite` |
| `vobes-config` | `toml` |
| `vobes-cli` | `clap`, `console`, `comfy-table`, `indicatif` |
| `vobes-desktop` | `tauri`, `tauri-build` (build) |

## Frontend

To be added at Phase 8. Likely:

- Svelte or React
- Vite
- `@tauri-apps/api`
- A small component library or none (hand-built minimal UI)

## Why Bundled SQLite

`rusqlite` with `features = ["bundled"]` statically links SQLite.
No system dependency. Single binary works on any system.
