# Vobes

A developer command center.

Vobes unifies fragmented developer context — git status, recent commits,
project health, package managers, frameworks, local notes, TODOs, build
status, project metadata — into a single calm workspace.

A **vobe** is one software project managed by Vobes.

- **CLI**: `vbs`
- **Desktop**: Vobes
- **One core, two faces.**

## Status

Pre-alpha. Phase 0 scaffolding. See [`design/`](./design/) for the full
plan.

## Quick start

```bash
cargo build -p vobes-cli
cargo run -p vobes-cli -- --help
```

### Configure

```bash
vbs init                     # writes ~/.config/vobes/config.toml
$EDITOR ~/.config/vobes/config.toml   # edit scan roots
```

### Use

```bash
vbs scan             # discover projects in configured roots
vbs list             # show tracked vobes (status table)
vbs show <name>      # detailed view of one vobe
vbs log              # recent activity across all vobes
vbs open <name>      # mark opened + launch $EDITOR
vbs sync             # re-scan roots, refresh git cache, record activity
vbs add <path>       # manually track a project
vbs rm <name>        # untrack a vobe
vbs export           # dump all data as JSON to ~/.vobes/snapshots/
```

## Design

The full product design lives in [`design/`](./design/README.md).

## License

MIT — (c) Yizhou Jiang