# Scanning

The application should be able to understand projects on the local
machine.

## Goals

- scan configured directories
- recognize repositories
- detect frameworks
- identify languages
- collect metadata
- keep the system modular

## Modular Detection

Each detector is a small unit behind a trait:

```rust
trait Detector {
    fn name(&self) -> &str;
    fn detect(&self, path: &Path, ctx: &DetectionContext) -> Result<Option<Detection>>;
}
```

The scanner composes many detectors. Adding a new framework or language
means adding one detector — no core change.

## Detectors

### Repository Detector

Looks for:

- `.git/` directory
- `.git` file (worktree)
- submodule references

If found → `is_repo = true`, mark as candidate vobe.

### Language Detector

- file extension counts (`.rs`, `.ts`, `.py`, `.go`, `.swift`…)
- shebang lines (`#!/usr/bin/env python`)
- primary language = highest non-vendored count

### Framework Detector

Signature-based on manifest files:

| Manifest | Frameworks detected |
|---|---|
| `package.json` | Next.js, React, Vue, Svelte, Nuxt, Remix, Express, … |
| `Cargo.toml` | Axum, Actix, Rocket, Bevy, … |
| `pyproject.toml` | FastAPI, Django, Flask, Poetry, … |
| `go.mod` | Gin, Echo, Fiber, … |
| `*.xcodeproj` | (Apple platforms) |
| `pubspec.yaml` | Flutter |
| `mix.exs` | Phoenix |

Each detector inspects dependencies and reports one (or zero) frameworks.

### Package Manager Detector

| File present | Package manager |
|---|---|
| `pnpm-lock.yaml` | pnpm |
| `yarn.lock` | yarn |
| `package-lock.json` | npm |
| `Cargo.lock` | cargo |
| `poetry.lock` | poetry |
| `Pipfile.lock` | pipenv |
| `go.sum` | go modules |
| `Gemfile.lock` | bundler |
| `mix.lock` | hex |

## Scan Flow

```
roots (from config)
   ↓
walk directories (respect exclude + max_depth)
   ↓
for each candidate:
   ↓
   run all detectors in parallel
   ↓
   merge detections → candidate vobe
   ↓
   compare with stored vobe
   ↓
   update / insert / leave alone
```

## Excludes

Built-in excludes (always applied):

- `node_modules`
- `.git` (we detect it, don't descend)
- `target`
- `dist`
- `build`
- `.cache`
- `vendor`
- `.next`
- `.venv`

User can extend via `vobes.toml`.

## Performance

- Parallel directory walk (rayon).
- Detector results cached by path + mtime.
- Incremental sync: only re-scan changed directories.
