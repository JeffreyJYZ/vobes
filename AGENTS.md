# AGENTS

## Architecture

- **One core, two faces.** `crates/vobes-core` shared; `vobes-cli` +
  `vobes-desktop` both link it. No CLI ↔ desktop IPC — both read same SQLite
  store + `config.toml`. Do not introduce IPC.
- **Shared state.** SQLite at platform data dir
  (`~/Library/Application Support/vobes/vobes.db` on macOS, etc.). Config via
  `~/Library/Application Support/vobes/config.toml`. Debug builds suffix `-dev`.
  Override with `VOBES_APP_DIR`.
- **Workspace query language.** `tag:X`, `lang:X`, `fw:X`, `pm:X`, `is:<flag>`,
  `name:X`. Predicate parsing in `desktop/src/lib/stores.ts` (`parseQuery`). Keep
  CLI + desktop search semantically aligned.
- **Plugin surface.** `crates/vobes-core/src/plugins.rs` defines trait; new
  capabilities extend core, not desktop.

## Style

- **Rust.** `cargo fmt` + `cargo clippy --all-targets -- -D warnings`. CI strict.
  Run both before commit.
- **Frontend.** `svelte-check` (0 errors target). `biome check` exists but
  **not** gated in CI — repo has pre-existing format debt
  (`semicolons: "asNeeded"` vs in-file semicolons). Do not reformat whole tree;
  match surrounding file's style.
- **Commits.** Conventionalish, subject ≤ 50–70 chars. Body only when "why"
  isn't obvious. No emojis in code or commits.
- **No comments** unless asked.
- **Do not add comments, docstrings, or tests user did not ask for.**

## Committing

- **Commit only, do not push.** User pushes by hand.
- One focused change per commit. Split when scope drifts.
- Format:
  ```
  feat(<scope>): <what>
  
  <why, if not obvious>
  ```
- Never amend a commit user has already seen unless explicitly asked.

## Build & verify

```bash
# core (CI-gated)
cargo fmt --all -- --check
cargo clippy --workspace --exclude vobes-desktop --all-targets -- -D warnings
cargo test  --workspace --exclude vobes-desktop

# desktop (CI-gated)
cd desktop
pnpm install --frozen-lockfile
pnpm build                                # frontend bundle
cargo clippy -p vobes-desktop --all-targets -- -D warnings
cargo build  -p vobes-desktop             # tauri shell

# quick front-end type check
npx svelte-check --tsconfig ./tsconfig.json
```

Always run relevant checks before claiming task done.

## Release flow

Every crate carries own `version = "X.Y.Z"` literal — no shared workspace
version. CLI tools (`vobes-cli`, `vobes-mcp`) + desktop (`vobes-desktop`, plus
`desktop/package.json` + `desktop/src-tauri/tauri.conf.json`) each version
independently. Bump only crates that changed.

To bump a crate:

1. Edit its `version =` line in own `Cargo.toml`.
2. If another workspace crate depends on it via `workspace = true`, also bump
   matching `version =` line in root `[workspace.dependencies]` table so
   `cargo publish` resolves.
3. Tag `vX.Y.Z` only when **desktop** version changed. Crates.io-only bumps need
   no tag — `cargo publish -p <name>` enough.

`.github/workflows/release.yml` creates **draft** GitHub release named
`Vobes vX.Y.Z Pre-alpha` — user publishes manually. `quality` job
(`fmt + clippy + test`, `--exclude vobes-desktop`) gates release build.

## Schema migrations

- Bump `SCHEMA_VERSION` in `crates/vobes-store/src/schema.rs` whenever migration
  runs. Add migration step in `migrate()`.
- Migrations run on open; existing versions table tracks applied migrations.
  Never drop/rename column without migration.

## Activity / agents

- `VOBES_ACTOR` env var tags every `ActivityEvent` with actor (`human`,
  `agent:claude`, etc.). Read via `now_env()`.
- MCP server is `crates/vobes-mcp` (JSON-RPC over stdio). Tools: `vobes_list`,
  `vobes_show`, `vobes_search`, `vobes_recent_activity`, `vobes_context`.

## Layout

```
crates/
  vobes-core/      // shared types, traits, error, plugin surface
  vobes-store/     // SQLite + schema migrations
  vobes-scan/      // project detection (Cargo, package.json, etc.)
  vobes-git/       // git status / branch / ahead-behind
  vobes-config/    // config.toml loader
  vobes-mcp/       // MCP stdio server
  vobes-cli/       // the `vbs` binary
desktop/
  src/             // Svelte frontend
    components/    // Select, Toast, …
    views/         // Dashboard, Projects, Activity, Settings
    lib/           // api, stores, format, markdown, …
  src-tauri/       // Rust shell, commands/, platform.rs (terminal/editor)
```

## Don'ts

- Don't push unless very explicit message from user. Commit only.
- Don't reformat whole tree to biome's house style.
- Don't add tests, doc-comments, or refactors user didn't request.
- Don't introduce build-time codegen pipeline unless asked.
- Don't break desktop ↔ CLI parity contract without flagging it in parity table
  in `README.md`.
- Always prefer ripgrep over grep, when applicable.
