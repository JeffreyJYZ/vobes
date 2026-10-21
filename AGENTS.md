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
  `name:X`. The parser lives in `crates/vobes-core/src/query.rs` (`Query::parse`) —
  the Rust side is the source of truth, and `desktop/src/lib/stores.ts` mirrors it.
  Every face uses it: CLI `vbs list --query`, MCP `vobes_search`, desktop search.
  Change one, change the other.
- **Config file is user-owned.** `config.toml` keeps the user's comments; the
  loader prunes stale keys with `toml_edit` rather than rewriting the file.
  Never rewrite it from the canonical shape.
- **Plugin surface.** `crates/vobes-core/src/plugins.rs` defines trait; new
  capabilities extend core, not desktop.

## Style

- **Rust.** `cargo fmt` + `cargo clippy --all-targets -- -D warnings`. CI strict.
  Run both before commit.
- **Frontend.** `svelte-check` (0 errors target) and `biome check` (clean).
  Biome formats TS/JS/JSON only: its Svelte formatter de-indents everything
  inside `<script>` to column 0, so `**/*.svelte` has the formatter disabled
  in `desktop/biome.json` (linting still on). Don't re-enable it.
  Either way `bun run lint` must stay at zero errors.
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
bun install --frozen-lockfile
bun run build                             # frontend bundle
cargo clippy -p vobes-desktop --all-targets -- -D warnings
cargo build  -p vobes-desktop             # tauri shell

# quick front-end type check
bunx svelte-check --tsconfig ./tsconfig.json

# front-end unit tests (bun:test, not vitest)
bun test
```

Always run relevant checks before claiming task done.

## Auto-update

- Desktop ships `tauri-plugin-updater` + `@tauri-apps/plugin-updater`; endpoint
  is `releases/latest/download/latest.json`, pubkey lives in
  `desktop/src-tauri/tauri.conf.json`.
- Release builds must be signed: repo secret `TAURI_SIGNING_PRIVATE_KEY` holds
  `~/.tauri/vobes-updater.key` (unencrypted, no password secret). Unsigned
  builds are rejected by every installed copy.
- **Never regenerate that keypair once a signed release is public** — installed
  copies pin the pubkey, so a new key strands them.
- The public key is committed at `desktop/src-tauri/vobes-updater.pub`; CI
  asserts it matches the `pubkey` pinned in `tauri.conf.json`. A mismatch means
  the updater rejects every signed build.
- `bump.yml` publishes bumped crates and skips versions already on the
  registry; `release.yml` signs and uploads `latest.json` + `.sig` artifacts.
  Crates publish **before** the tag is cut, so the release notes never
  advertise a version that is not on crates.io yet.
- Secret `CARGO_REGISTRY_TOKEN` is required by `bump.yml`.

## Release flow

Every crate carries own `version = "X.Y.Z"` literal — no shared workspace
version. CLI tools (`vobes-cli`, `vobes-mcp`) + desktop (`vobes-desktop`, plus
`desktop/package.json` + `desktop/src-tauri/tauri.conf.json`) each version
independently. Bump only crates that changed.

To bump a crate:

1. Edit its `version =` line in own `Cargo.toml`.
2. If another workspace crate depends on it via `workspace = true`, also bump
   matching `version =` line in root `[workspace.dependencies]` table so
   `cargo publish` resolves. CI fails the build on a mismatch — a member
   bumped without the root table publishes a crate pinned to the OLD sibling
   version, and crates.io versions are immutable.
3. Bump the desktop version in **all three** places: `desktop/src-tauri/Cargo.toml`,
   `desktop/package.json`, `desktop/src-tauri/tauri.conf.json`. The tag comes
   from `tauri.conf.json` (the updater compares that one) and CI asserts all
   three agree.
4. Tag `vX.Y.Z` only when **desktop** version changed. Crates.io-only bumps need
   no tag — `cargo publish -p <name>` enough.

`.github/workflows/release.yml` creates **draft** GitHub release named
`Vobes vX.Y.Z Pre-alpha` — user publishes manually. `quality` job
(`fmt + clippy + test`, `--exclude vobes-desktop`, plus the desktop clippy and
the frontend lint/typecheck/test) gates the release build.

- **A tag pushed by `GITHUB_TOKEN` never triggers another workflow** (GitHub's
  anti-recursion rule), so `bump.yml` must `gh workflow run release.yml --ref
  "$TAG"` after tagging — otherwise the tag lands and no build ever starts
  (cost v0.1.10). Pushing the tag by hand from a laptop *does* trigger it.
- `bump.yml` publishes bumped crates and skips versions already on the
  registry; needs secret `CARGO_REGISTRY_TOKEN`.
- The publish runs **before** the tag, so the release notes never advertise a
  crate version that is not on crates.io yet.
- A crate publish that fails does not stop the others: each crate is isolated
  and the failures are listed at the end.
- A crate version bump is only published when the desktop version moved too —
  crates.io versions are immutable, so a publish cannot be undone.

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
