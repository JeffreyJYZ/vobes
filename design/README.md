# Vobes Design

Vobes is a developer command center — unifying project management, Git
awareness, activity tracking, and local knowledge into one calm, fast,
keyboard-friendly workspace.

A **vobe** is one software project managed by Vobes.

- CLI: `vbs`
- Desktop: Vobes
- Same core, two faces.

## Menu

### Vision & Principles

- [Overview](./01-overview.md) — what Vobes is
- [Product Ethos](./02-ethos.md) — quality over quantity, cut features
- [User Experience](./03-ux.md) — minimal, calm, fast, native
- [Architecture Goals](./04-architecture.md) — boundaries, no bloat

### Concepts

- [Vobe (Data Model)](./10-vobe.md) — one project, all its metadata
- [Activity](./11-activity.md) — recently opened, modified, timeline
- [Git Awareness](./12-git.md) — first-class Git info, no replacement

### System

- [Shared Core](./20-shared-core.md) — one core, two faces
- [Platforms](./21-platforms.md) — desktop + CLI
- [Scanning](./22-scanning.md) — dirs, repos, frameworks, languages
- [Configuration](./23-configuration.md) — human-readable, extensible
- [Storage](./24-storage.md) — SQLite + JSON export
- [Technology](./25-technology.md) — stack choices

### Build

- [Repo Structure](./30-repo-structure.md) — monorepo layout
- [Phases](./31-phases.md) — execution order
- [Dependencies](./32-dependencies.md) — Rust crate map

### Future

- [Cross-Platform](./40-cross-platform.md) — macOS, Linux, Windows
- [AI Integration](./41-ai-integration.md) — agent surface

## Quick Facts

| | |
|---|---|
| Stack | Rust + Tauri |
| Repo | Monorepo (Cargo workspace) |
| Storage | SQLite + JSON export |
| Config | TOML |
| OS | Cross-platform from day one |
| First ship | Core + scanning + CLI |

## North Star

> "I want to manage all of my projects here."

If a choice compromises architecture or user experience, remove the feature instead.
