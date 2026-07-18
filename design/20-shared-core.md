# Shared Core

The most important architectural requirement.

> One shared core. Business logic must not be duplicated between desktop
> and CLI.

## What Lives in Core

- **Models** — Vobe, ActivityEvent, GitInfo, Commit
- **Scanning** — directory walk, repo/framework/language detection
- **Git interactions** — branch, dirty, ahead/behind, log
- **Storage** — SQLite + JSON export
- **Configuration** — loading, defaults, validation
- **Activity** — event recording, timeline queries
- **Errors** — typed errors shared across all consumers

## What Stays Platform-Specific

- **CLI** — argument parsing, terminal output, table formatting
- **Desktop** — Tauri commands, UI rendering, frontend state
- **OS paths** — `dirs` crate resolution
- **Launchers** — opening editors, opening URLs

## Crates

| Crate | Responsibility |
|---|---|
| `vobes-core` | Domain models, traits, error types |
| `vobes-scan` | Scanning engine, detectors |
| `vobes-git` | Git status, log, branch info |
| `vobes-activity` | Event recording, timeline queries |
| `vobes-store` | SQLite + JSON export, migrations |
| `vobes-config` | Configuration loading, defaults |
| `vobes-cli` | CLI binary (`vbs`) |
| `vobes-desktop` | Tauri app (backend only; frontend in `desktop/`) |

## Dependency Rule

```
vobes-cli  ─┐
            ├──> vobes-core
vobes-desktop ┘        │
            ├──────────┤
            │  + scan, git, activity, store, config
```

- `vobes-core` depends on nothing internal.
- Platform crates (`cli`, `desktop`) depend on everything but never
  the other way around.
- No circular dependencies.
- No platform crate may be imported by a shared crate.

## Trait Boundaries

Each subsystem exposes a trait so the platform can swap implementations
(test mocks, alternative storage, etc.):

```rust
// in vobes-store
trait Store {
    fn upsert_vobe(&self, vobe: &Vobe) -> Result<()>;
    fn get_vobe(&self, id: &VobeId) -> Result<Option<Vobe>>;
    fn list_vobes(&self, filter: &Filter) -> Result<Vec<Vobe>>;
    // ...
}

// in vobes-scan
trait Scanner {
    fn scan(&self, root: &Path) -> Result<ScanReport>;
}
```

## Future

- MCP server crate (shares core)
- Web API crate (shares core)
- Plugin/extension crate (shares core)
