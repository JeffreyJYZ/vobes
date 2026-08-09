# Vobes

A developer command center.

Vobes unifies fragmented developer context — git status, recent commits,
project health, package managers, frameworks, local notes, project
metadata — into a single calm workspace.

A **vobe** is one software project managed by Vobes.

- **CLI**: `vbs`
- **Desktop**: Vobes
- **One core, two faces.**

## Status

Pre-alpha. Core, scanning, git, activity, SQLite store, config, CLI,
desktop (with command palette, file watcher, deep links, notifications),
and an MCP server for AI agents. See [`ROADMAP.md`](./ROADMAP.md) for the
current UX roadmap.

> Dev and release builds use separate data directories
> (`vobes-dev` vs `vobes`) so `cargo tauri dev` can't overwrite an
> installed copy. Set `VOBES_APP_DIR` to override.

## Quick start (CLI)

```bash
cargo build -p vobes-cli
cargo run -p vobes-cli -- --help
```

### Configure

```bash
vbs init                              # writes config.toml in your config dir
$EDITOR "$(vbs init 2>&1 | tail -1)" # edit scan roots (path is printed)
```

On macOS the config lives at `~/Library/Application Support/vobes/config.toml`,
on Linux `~/.config/vobes/config.toml`, on Windows `%APPDATA%\vobes\config.toml`.
Debug builds use a `-dev` suffix.

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
vbs export           # dump all data as JSON to the snapshots dir
```

### For AI agents

Vobes exposes its data to agents in three ways:

```bash
vbs list --json        # machine-readable vobe list
vbs show <name> --json # machine-readable vobe detail
vbs log --json         # machine-readable activity
vbs context <name>     # full context pack (record + activity + dir entries)
vbs watch              # stream activity as NDJSON
```

Or run the MCP server (JSON-RPC 2.0 over stdio):

```bash
cargo run -p vobes-mcp
```

Tools: `vobes_list`, `vobes_show`, `vobes_search`, `vobes_recent_activity`, `vobes_context`.

## Desktop (Tauri)

The desktop app wraps the same core as the CLI — same config file, same
SQLite store, same scanner — and adds a keyboard-first UI.

Highlights:
- **Command palette** (`⌘K` / `Ctrl+K`) — fuzzy-find vobes and run any action
- **Global shortcut** (`Ctrl+Alt+V`) — summon the palette from any app
- **Attention section** — dirty repos, unpushed, behind upstream at a glance
- **Project detail** — git state, last commit, notes, README preview, TODO scrape, context-pack copy
- **File watcher** — incremental refresh when files change
- **Deep links** — `vobes://open/<name>` and `vobes://search?q=…`
- **Notifications** — opt-in alert when a vobe falls behind upstream
- **Saved views** — pin a search to the sidebar
- **In-app settings** — edit roots, theme, git cache without touching TOML

### Prerequisites

- Rust stable
- Node 22+ and pnpm 11+
- macOS: Xcode CLI tools
- Linux: `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-app_indicator3-dev
  librsvg2-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev`
- Windows: WebView2 runtime (preinstalled on Windows 11)

### Build

```bash
cd desktop
pnpm install
cargo tauri dev        # hot-reload dev loop (frontend + rust)
cargo tauri build      # produce installable bundle (in desktop/src-tauri/target/release/bundle)
```

## License

MIT — (c) Yizhou Jiang