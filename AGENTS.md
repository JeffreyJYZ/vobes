# AGENTS

Operating manual for AI coding agents working on Vobes.

## Architecture

- **One core, two faces.** `crates/vobes-core` is shared; `vobes-cli` and
  `vobes-desktop` both link it. No CLI ↔ desktop IPC — they read the same
  SQLite store and `config.toml`. Do not introduce IPC.
- **Shared state.** SQLite at the platform-specific data dir
  (`~/Library/Application Support/vobes/vobes.db` on macOS, etc.). Config
  via `~/Library/Application Support/vobes/config.toml`. Debug builds
  suffix `-dev`. Override with `VOBES_APP_DIR`.
- **Workspace query language.** `tag:X`, `lang:X`, `fw:X`, `pm:X`,
  `is:<flag>`, `name:X`. Predicate parsing lives in
  `desktop/src/lib/stores.ts` (`parseQuery`). Keep CLI and desktop
  search semantically aligned.
- **Plugin surface.** `crates/vobes-core/src/plugins.rs` defines the
  trait; new capabilities extend core, not desktop.

## Style

- **Rust.** `cargo fmt` + `cargo clippy --all-targets -- -D warnings`.
  CI is strict. Run both before committing.
- **Frontend.** `svelte-check` (0 errors target). `biome check` exists
  but is **not** gated in CI — repo has pre-existing format debt
  (`semicolons: "asNeeded"` vs in-file semicolons). Do not reformat the
  whole tree to fix it; match the surrounding file's style.
- **Commits.** Conventionalish, subject ≤ 50–70 chars. Body only when
  "why" isn't obvious. No emojis in code or commits.
- **No comments** unless asked. Code should speak for itself.
- **Do not add comments, docstrings, or tests the user did not ask for.**

## Committing

- **Commit only, do not push.** The user pushes by hand.
- One focused change per commit. Split when scope drifts.
- Format:
  ```
  feat(<scope>): <what>
  
  <why, if not obvious>
  ```
- Never amend a commit the user has already seen unless explicitly
  asked.

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

Always run the relevant checks before claiming a task is done.

## Release flow

Every crate carries its own `version = "X.Y.Z"` literal — there is no
shared workspace version. CLI tools (`vobes-cli`, `vobes-mcp`) and the
desktop (`vobes-desktop`, plus `desktop/package.json` and
`desktop/src-tauri/tauri.conf.json`) each version independently. Bump
only the crates that actually changed.

To bump a crate:

1. Edit its `version =` line in its own `Cargo.toml`.
2. If another workspace crate depends on it via `workspace = true`,
   also bump the matching `version =` line in the root `[workspace.dependencies]`
   table so `cargo publish` resolves.
3. Tag with `vX.Y.Z` only when the **desktop** version changed.
   Crates.io-only bumps don't need a tag — `cargo publish -p <name>`
   is enough.

`.github/workflows/release.yml` creates a **draft** GitHub release
named `Vobes vX.Y.Z Pre-alpha` — the user publishes manually. A
`quality` job (`fmt + clippy + test`, `--exclude vobes-desktop`) gates
the release build.

## Schema migrations

- Bump `SCHEMA_VERSION` in `crates/vobes-store/src/schema.rs` whenever
  a migration runs. Add a migration step in `migrate()`.
- Migrations run on open; the existing versions table tracks applied
  migrations. Never drop or rename a column without a migration.

## Activity / agents

- `VOBES_ACTOR` env var tags every `ActivityEvent` with its actor
  (`human`, `agent:claude`, etc.). Read it via `now_env()`.
- MCP server is `crates/vobes-mcp` (JSON-RPC over stdio). Tools:
  `vobes_list`, `vobes_show`, `vobes_search`,
  `vobes_recent_activity`, `vobes_context`.

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

- Don't push. Commit only.
- Don't reformat the whole tree to biome's house style.
- Don't add tests, doc-comments, or refactors the user didn't request.
- Don't introduce a build-time codegen pipeline unless asked.
- Don't break the desktop ↔ CLI parity contract without flagging it in
  the parity table in `README.md`.