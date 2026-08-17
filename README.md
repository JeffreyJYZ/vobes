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
and an MCP server for AI agents. See the CLI ↔ desktop parity
table below for what each face supports.

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
- **Split-button openers** — launch any vobe in the user's chosen terminal or editor; defaults stay sticky per-app
- **Set-as-default star** in the terminal/editor dropdown — pin a favourite without leaving the menu; persisted across sessions
- **Persistent error toasts** with a close `×` so failures don't flash past you
- **Reset and rescan** from the dashboard when the cached project list drifts
- **Project detail** — git state, last commit, notes, README preview, TODO scrape, context-pack copy
- **File watcher** — incremental refresh when files change
- **Deep links** — `vobes://open/<name>` and `vobes://search?q=…` (registered by the installed bundle; not active in `cargo tauri dev`)
- **Notifications** — opt-in alert when a vobe falls behind upstream
- **Saved views** — pin a search to the sidebar
- **In-app settings** — edit roots, theme, git cache, snapshot history without touching TOML

### First run

The desktop ships with an empty Dashboard. Configure scan roots in
**Settings → Roots**, then click **Reset and rescan** (or hit
`⌘K` → "Rescan") to populate it. The same `config.toml` is shared with
the CLI.

### Search syntax

The command palette and Dashboard search accept predicates prefixed by a
key (case-insensitive, whitespace-tolerant):

- `tag:rust` — only vobes tagged `rust`
- `lang:rust` / `fw:react` / `pm:pnpm` — language, framework, package manager
- `is:dirty` / `is:behind` / `is:pinned` / `is:attention` — git/pin flags
- `name:foo` — name substring; bare words are also fuzzy-matched

A saved filter is just a named query — pin it to the sidebar for a
personal workspace scoped to a tag, language, or attention slice.

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

### Auto-update

The updater is wired (`tauri-plugin-updater` + `createUpdaterArtifacts: true`,
endpoint pointing at GitHub `latest.json`) but **inert** until a signing key
is configured. Without it, every release fails signature verification and
the running app silently skips the update.

One-time setup:

1. Generate a keypair (private key stays local; public key goes in the repo):
   ```bash
   pnpm tauri signer generate -w ~/.tauri/vobes.key
   ```
2. Paste the contents of `vobes.key.pub` into `plugins.updater.pubkey` in
   `desktop/src-tauri/tauri.conf.json` (replaces the
   `REPLACE_WITH_TAURI_SIGNER_GENERATE_OUTPUT` placeholder).
3. Add the private key as a GitHub Actions secret:
   ```bash
   gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/vobes.key
   # optional, only if you set a password during generate:
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD <password>
   ```
   The release workflow already reads `secrets.TAURI_SIGNING_PRIVATE_KEY`;
   uncomment the env lines in `.github/workflows/release.yml` when ready.

Until then, builds still work, releases still publish as drafts, but users
on older versions get no update prompt.

### Crash reporting

Panics and unhandled errors flow to a Sentry-compatible endpoint
(`sentry` Rust crate + `@sentry/svelte` frontend). Both sides read their
DSN from an env var and no-op when unset, so dev builds stay local.

The Tauri side doesn't care which Sentry-compatible server you point at
— the only requirement is a reachable DSN URL. Self-hosting options:

- **GlitchTip** (Sentry-API-compatible) pointed at any Postgres. The
  Vobes stack assumes GlitchTip + Postgres, but you can also point at
  Sentry SaaS, Sentry self-hosted, Highlight, or any other compatible
  endpoint.

For the Postgres behind GlitchTip, any provider works — the
`DATABASE_URL` env var accepts any libpq-compatible connection string:

| Provider | Notes |
|---|---|
| Neon | Free tier 0.5 GB, AWS-backed. Variable reachability from mainland China. |
| Aliyun RDS for PostgreSQL / PolarDB | China-hosted, ICP-friendly. Recommended for China users. |
| Tencent Cloud TDSQL-C / PostgreSQL | Same — China-hosted, low latency from CN. |
| Supabase | Hosted Postgres + dashboard. Good if you also want auth later. |
| Local Docker | `docker run -e POSTGRES_PASSWORD=... postgres:16`. Fine for self-hosting. |

If your users are in mainland China and Neon/Sentry SaaS feel slow, host
GlitchTip on an Aliyun ECS or Tencent CVM instance in the same region
as the Postgres.

One-time setup:

1. Provision GlitchTip + Postgres. GlitchTip needs `DATABASE_URL`
   pointing at the Postgres, `SITE_URL`, `SECRET_KEY` (any random
   string), and a public hostname. Docker example:
   ```bash
   docker run -d --name glitchtip \
     -e DATABASE_URL=postgres://glitchtip:secret@postgres:5432/glitchtip \
     -e SITE_URL=https://glitchtip.your-domain.com \
     -e SECRET_KEY=$(openssl rand -hex 32) \
     -p 8000:8000 \
     glitchtip/glitchtip
   ```
2. Create a project in GlitchTip, copy the DSN (format
   `https://<key>@<host>/<project-id>`).
3. Set the DSN at build time:
   ```bash
   # Rust side (panic hook)
   export VOBES_SENTRY_DSN='https://...@.../...'
   # Frontend (init in main.ts)
   export VITE_SENTRY_DSN='https://...@.../...'
   ```
4. For production builds, add both env vars to the release workflow
   env block (`.github/workflows/release.yml`).

Until the DSN is set, crashes still print to stderr + show in the
"fatal" overlay — nothing leaves the machine.

## CLI ↔ desktop parity

Both faces talk to the same core. Capabilities should match; gaps are bugs.

| Capability | CLI (`vbs`) | Desktop | Notes |
|---|---|---|---|
| Scan roots, discover projects | `scan`, `sync` | `scan`, `sync` commands | identical detector pipeline |
| List vobes | `list`, `list --json` | dashboard grid | desktop adds fuzzy search + sort UI |
| Show one vobe | `show <name>`, `show --json` | Projects view | desktop adds README/TODO/notes |
| Recent activity | `log`, `log --json` | Activity view | both filter by actor |
| Activity per vobe | `show` (embedded) | Projects activity card | — |
| Add vobe | `add <path>` | "Add vobe…" | — |
| Remove vobe | `rm <name>` | "Remove" in Projects view | — |
| Open in editor | `open <name>` | "Open in editor" | desktop uses shell plugin |
| Export JSON snapshot | `export` | Settings → Snapshots → Export now | both write `snapshots_dir/vobes-<ts>.json` |
| Restore snapshot | `import` (planned) | Settings → Snapshots → Restore | CLI import pending — see `Store::import_json` |
| Pin vobe | — (set via store) | pin button, `set_pinned` | desktop-only surface for now |
| Tags | — (set via store) | Projects view tag editor | desktop-only surface for now |
| Notes | — (set via store) | Projects notes editor | desktop-only surface for now |
| Saved filters | — | sidebar + `save_saved_filter` | desktop-only surface for now |
| Workspaces (tag scope) | — | sidebar tag click, `tag:X` query | desktop-only surface for now |
| Agent attribution | `VOBES_ACTOR=agent vbs …` | inherits env | `ActivityEvent::now_env` |
| MCP tools | `vobes-mcp` binary | — | `vobes_list/show/search/recent_activity/context` over stdio |
| Reset + rescan | `reset --yes` | "Reset and rescan" | both call `Store::purge_all` |
| Pick terminal app | — | split-button selector | `list_terminals` + `open_terminal_with` |
| Pick editor app | — | split-button selector | `list_editors` + `open_in_editor` |
| Persistent defaults | — | star in terminal/editor dropdown | localStorage (`vobes:default-terminal/editor`) |
| Snapshot history UI | — | Settings → Snapshots list + Restore | desktop-only surface; CLI `import` pending |

Where the CLI is missing a surface that exists in the desktop (saved
filters, workspaces, pin/tags/notes CLI commands), it's tracked as
follow-up work, not a permanent split.

## License

MIT — (c) Yizhou Jiang