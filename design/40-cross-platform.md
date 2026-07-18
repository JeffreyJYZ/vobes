# Cross-Platform

Cross-platform from day one. Target: macOS, Linux, Windows.

## Strategy

- All Rust code is portable. No `cfg(target_os = "...")` unless required.
- All paths via `dirs` crate — never `~` strings at use sites.
- All terminals via cross-platform crates.
- Git via `git2` — works on all 3 OSes.
- Tauri supports all 3 OSes.
- SQLite is embedded and works on all 3.

## Platform Paths

The `dirs` crate resolves:

| Resource | Path (per OS) |
|---|---|
| User config | `~/.config/vobes/config.toml` (Linux), `~/Library/Application Support/vobes/config.toml` (macOS), `%APPDATA%\vobes\config.toml` (Windows) |
| Database | same parent dir, `vobes.db` |
| Exports | same parent dir, `snapshots/` |

## CI Matrix

GitHub Actions:

```yaml
strategy:
  matrix:
    os: [macos-latest, ubuntu-latest, windows-latest]
```

Jobs:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --release` (sanity)

## Tauri Build Matrix

Phase 8 adds Tauri builds per OS. Tauri handles most of this; we just
provide icons per platform and ensure the frontend builds in CI.

## OS-Specific Tests

- Path resolution tests per OS (mocked if needed via `cfg`).
- File permission tests skipped where not portable.
- Newline handling: always use `\n` in Rust strings; let writers
  translate if needed (we don't write text files directly).

## Known Constraints

- **Windows**: long path support (>260 chars) requires manifest.
  Defer; document the limit.
- **Linux**: X11 vs Wayland. Tauri/WebKit handles this.
- **macOS**: notarization + signing for distribution. Defer to
  release phase.

## Out of Scope (for now)

- Mobile.
- Web-based Vobes (vs Tauri app).
- Linux package distribution (deb, rpm, AUR). Manual binary for now.
