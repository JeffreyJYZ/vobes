# Repo Structure

Monorepo with a Cargo workspace.

```
vobes/
├── Cargo.toml              # Workspace root
├── vobes.toml              # Project-level config example
├── README.md
├── LICENSE
├── crates/
│   ├── vobes-core/         # Models, traits, errors
│   ├── vobes-scan/         # Scanning engine
│   ├── vobes-git/          # Git interactions
│   ├── vobes-activity/     # Activity tracking
│   ├── vobes-store/        # SQLite + JSON
│   ├── vobes-config/       # Config loading
│   ├── vobes-cli/          # CLI binary (vbs)
│   └── vobes-desktop/      # Tauri backend
├── desktop/                # Frontend (Tauri)
│   ├── src/                # UI
│   └── src-tauri/          # Tauri shell config
├── docs/
└── design/                 # ← you are here
```

## Crate Purposes

### `vobes-core`

- `Vobe`, `VobeId`, `ActivityEvent`, `GitInfo`, `Commit` types
- `Filter`, `Sort`, `Result` aliases
- Shared error types
- No filesystem, no network, no IO. Pure models + traits.

### `vobes-scan`

- `Scanner` trait + default impl
- `Detector` trait + per-framework impls
- Repo/framework/language/package-manager detection
- Excludes, max depth, parallel walk

### `vobes-git`

- `GitStatus` queries via `git2`
- Branch, dirty, ahead/behind, last commit
- Read-only

### `vobes-activity`

- Event recording
- Timeline queries
- Hooks for future event sources

### `vobes-store`

- `Store` trait
- `SqliteStore` impl
- `JsonExporter` impl
- Migrations

### `vobes-config`

- Config structs (TOML deserialization)
- Default values
- `~` expansion
- Path resolution

### `vobes-cli`

- `clap` argument parsing
- Command implementations
- Output formatting (table, json, plain)
- Subcommands: `scan`, `list`, `show`, `log`, `sync`, `add`, `rm`,
  `open`, `export`, `init`, `desktop`

### `vobes-desktop`

- Tauri command handlers
- Wires frontend ↔ core
- No UI logic

## Workspace `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/*",
]

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
clap = { version = "4", features = ["derive"] }
git2 = "0.18"
rusqlite = { version = "0.31", features = ["bundled"] }
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
console = "0.15"
comfy-table = "7"
indicatif = "0.17"
rayon = "1.10"
dirs = "5"
walkdir = "2"
```

## Frontend

TBD. Either Svelte (smaller, simpler) or React (larger ecosystem).
Decide at Phase 8.

```
desktop/
├── src/
│   ├── routes/         # pages
│   ├── components/     # reusable
│   ├── lib/            # api clients
│   └── app.tsx|app.svelte
├── src-tauri/
│   ├── tauri.conf.json
│   └── Cargo.toml
└── package.json
```

## Git

- Single repo.
- `.gitignore` ignores: `target/`, `node_modules/`, `*.db`, `.DS_Store`.
- `main` is stable; `dev` is integration.

## CI (future)

GitHub Actions matrix: macOS, Linux, Windows.
Jobs: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`.
