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

Pre-alpha. Phases 0-10 implemented: core, scanning, git, activity, SQLite
store, config, CLI, desktop, and an MCP server for AI agents. See
[`design/`](./design/) for the full plan.

## Quick start (CLI)

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

### For AI agents

Vobes exposes its data to agents in three ways:

```bash
vbs list --json       # machine-readable vobe list
vbs show <name> --json # machine-readable vobe detail
vbs log --json        # machine-readable activity
vbs context <name>    # full context pack (record + activity + dir entries)
vbs watch             # stream activity as NDJSON
```

Or run the MCP server (JSON-RPC 2.0 over stdio):

```bash
cargo run -p vobes-mcp
```

Tools: `vobes_list`, `vobes_show`, `vobes_search`, `vobes_recent_activity`, `vobes_context`.

## Desktop (Tauri)

### Prerequisites

- Rust stable
- Node 20+ and pnpm 9+
- macOS: Xcode CLI tools
- Linux: `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
  librsvg2-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev`
- Windows: WebView2 runtime (preinstalled on Windows 11)

### Build

```bash
cd desktop
pnpm install
cargo tauri dev        # hot-reload dev loop (frontend + rust)
cargo tauri build      # produce installable bundle (in desktop/src-tauri/target/release/bundle)
```

The desktop app uses the same core as the CLI — same config file, same
SQLite store, same scanner.

## Design

The full product design lives in [`design/`](./design/README.md).

## License

MIT — (c) Yizhou Jiang