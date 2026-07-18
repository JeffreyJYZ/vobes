# Configuration

Configuration should be clean, human-readable, and easy to extend.

## Format

**TOML.** Native to Rust ecosystem, familiar to most developers, supports
nested tables cleanly.

## Files

### User Config

`~/.vobes/config.toml` (or platform-equivalent via `dirs`).

Per-user defaults, scan roots, display preferences.

### Project Config

`./vobes.toml` in a project root.

Per-project overrides. Optional.

## Example

```toml
[general]
name = "Personal Workspace"

[scan]
roots = ["~/dev", "~/work", "~/oss"]
exclude = ["scratch", "experiments"]
max_depth = 4
follow_symlinks = false

[display]
theme = "auto"
date_format = "relative"
default_sort = "last_modified"

[git]
cache_ttl_seconds = 60
fetch_upstream = false

[export]
path = "~/.vobes/snapshots"
format = "json"
```

## Design Rules

### Clean

- No duplicated keys.
- Sensible grouping via tables.
- Comments encouraged.

### Human-Readable

- Sentence-case keys.
- Self-describing values.
- No magic numbers without context.

### Easy to Extend

Adding a new section is just a new struct + `#[serde(default)]`.
Existing configs continue to work.

### Defaults Everywhere

Every field has a default. Empty config is a valid config.

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    general: GeneralConfig,
    #[serde(default)]
    scan: ScanConfig,
    #[serde(default)]
    display: DisplayConfig,
    #[serde(default)]
    git: GitConfig,
    #[serde(default)]
    export: ExportConfig,
}
```

### Future Expansion

Future additions are planned for, not crammed in:

- `[ai]` — AI integration settings
- `[plugins]` — plugin enablement
- `[hooks]` — event hooks (pre/post scan, on open, etc.)

These don't exist yet but the structure leaves room.
