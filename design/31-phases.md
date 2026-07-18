# Phases

Execution order.

```
Phase 0  ──>  Phase 1  ──>  Phase 2 + 3  ──>  Phase 4 + 5 + 6
                                                    │
                                                    v
                                            Phase 7 (CLI)
                                                    │
                                                    v
                                            Phase 8 (Desktop)
                                                    │
                                                    v
                                            Phase 9 (Polish)
                                                    │
                                                    v
                                           Phase 10 (AI)
```

## Phase 0 — Scaffold

- Cargo workspace
- Directory layout
- Root `vobes.toml` example
- License, README, CI stub
- `cargo check` succeeds with empty crates

## Phase 1 — Core Domain

- `vobes-core` with all model types
- Error types
- Traits (placeholder impls OK)
- Unit tests for serialization round-trip

## Phase 2 — Scanning

- `vobes-scan` crate
- `Scanner` trait + default impl
- Detectors: repo, language, framework, package manager
- Parallel walk with rayon
- Exclude rules
- Tests with fixture projects

## Phase 3 — Git

- `vobes-git` crate
- Read status via `git2`
- Caching
- Tests on fixture repos

## Phase 4 — Activity

- `vobes-activity` crate
- Event recording
- Timeline queries
- Hooks from `vbs open` and `vbs sync`

## Phase 5 — Storage

- `vobes-store` crate
- `Store` trait
- `SqliteStore` impl
- Migrations
- `JsonExporter`
- Tests against in-memory SQLite

## Phase 6 — Config

- `vobes-config` crate
- Config structs
- Defaults everywhere
- Path resolution
- Tests with sample TOML

## Phase 7 — CLI

- `vobes-cli` crate
- `clap` setup
- All subcommands wired
- Output formatting
- End-to-end test: scan, list, show, open, log, export

## Phase 8 — Desktop

- Tauri scaffold
- Frontend skeleton
- Wire Tauri commands to core
- Dashboard, project detail, timeline, settings

## Phase 9 — Cross-Platform Polish

- CI matrix: macOS, Linux, Windows
- Platform path tests
- Tauri builds for all 3 OS
- Bug fixes from real usage

## Phase 10 — AI Integration

- `vbs context <name>` command
- MCP server crate (uses core)
- Hooks for agent subscriptions
- Documentation for AI agent consumers

## What Each Phase Ships

| Phase | Shippable artifact |
|---|---|
| 0 | empty workspace, `cargo check` |
| 1 | models, no behavior |
| 2 | scanner library + tests |
| 3 | git library + tests |
| 4 | activity library + tests |
| 5 | storage library + tests |
| 6 | config library + tests |
| 7 | `vbs` binary (usable!) |
| 8 | Vobes desktop app |
| 9 | cross-platform CI green |
| 10 | AI agent surface |
