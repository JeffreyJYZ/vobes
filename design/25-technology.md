# Technology

Choose technologies that best support a modern desktop-first developer
tool.

The implementation details are less important than achieving a clean
architecture and an excellent developer experience.

## Stack

| Layer | Choice | Why |
|---|---|---|
| Language | **Rust** | Fast, memory-safe, single binary, cross-platform, perfect for dev tools |
| Desktop | **Tauri** | Rust backend, native shell, small bundle, modern |
| CLI | **Rust** | Same language as core, no runtime needed |
| Config | **TOML** | Human-readable, Rust-native |
| Storage | **SQLite + JSON** | Embedded primary, portable export |
| Frontend | TBD (React or Svelte) | Tauri-compatible, reactive |

## Why Rust

- One language across core, CLI, and desktop backend.
- Type safety catches model drift between platforms.
- Performance: instant startup, no GC pauses.
- Cross-platform: macOS, Linux, Windows from one codebase.
- Ecosystem: `git2`, `rusqlite`, `serde`, `clap` — all mature.

## Why Tauri

- Reuses our Rust backend directly. No IPC layer for shared logic.
- Smaller bundles than Electron.
- Native webview; no Chromium shipped.
- Easy frontend replacement.

## Why Not Electron

- Would force a second language split.
- Larger bundles.
- Memory footprint.

## Why Not Native (Swift/SwiftUI)

- Loses cross-platform from day one.
- Doubles maintenance.
- No CLI/agent story in the same language.

## Tradeoffs Accepted

- **Compile times** — Rust builds are slow. Mitigate with workspace layering and incremental compilation.
- **Web frontend** — Tauri uses web tech. We accept this in exchange for cross-platform UI velocity.
- **No mobile** — desktop only for now. Mobile not on the roadmap.

## What Could Change Later

- Frontend framework (React ↔ Svelte) — swappable inside Tauri.
- Storage backend (SQLite could be replaced by a remote one) — hidden behind the `Store` trait.
- Scanner implementations — added as new `Detector` impls.

The architecture is the contract. Tech choices inside it are replaceable.
