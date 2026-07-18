# Platforms

The ecosystem consists of two primary applications.

## Desktop

Vobes — the primary graphical experience.

### Responsibilities

- project dashboard
- visual project management
- activity timeline
- Git overview
- analytics
- project inspection
- settings
- future AI interface

### Feel

Modern, minimal, fast, native, keyboard-friendly.

### Stack

- **Tauri** for the shell.
- Frontend framework TBD (React or Svelte).
- Shared Rust backend with the CLI.

## CLI

`vbs` — the command-line interface.

### Principle

> Should feel like a natural extension of the desktop, not an unrelated
> utility.

### Capabilities

| Command | Purpose |
|---|---|
| `vbs scan` | discover projects in configured roots |
| `vbs list` | show all tracked vobes |
| `vbs show <name>` | inspect a vobe in detail |
| `vbs log` | activity timeline |
| `vbs sync` | re-scan, refresh git cache |
| `vbs add <path>` | manually add a vobe |
| `vbs rm <name>` | remove a vobe |
| `vbs open <name>` | record open + launch editor |
| `vbs export` | JSON dump |
| `vbs init` | create `vobes.toml` |
| `vbs desktop` | launch the desktop app |

### Stack

- **Rust** (shared with core)
- **clap** for argument parsing
- **console** + **comfy-table** for output
- **indicatif** for progress

## Two Faces, One System

The CLI and desktop must never diverge in behavior. If the desktop can
do something, the CLI can do it (modulo ergonomic constraints).

- Same storage → same data
- Same scanners → same detections
- Same config → same defaults
- Same git module → same git state
