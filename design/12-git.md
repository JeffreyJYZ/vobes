# Git Awareness

Git is a first-class concept in Vobes.

We do not attempt to replace Git. We **surface useful information**.

## Surfaced Fields

For every git project, Vobes tracks:

- **branch** — current branch name
- **dirty** — boolean, any unstaged or uncommitted changes
- **ahead** — commits ahead of upstream
- **behind** — commits behind upstream
- **last commit** — hash, message, author, date

## Data Model

```rust
struct GitInfo {
    branch: String,
    dirty: bool,
    ahead: u32,
    behind: u32,
    last_commit: Option<Commit>,
}

struct Commit {
    hash: String,
    message: String,
    author: String,
    date: DateTime<Utc>,
}
```

## Implementation

Use `git2` (libgit2 bindings). Cross-platform, no Git CLI dependency,
fast for the read-only operations we need.

### Read Operations

- `repo.head()` → branch
- `repo.index().is_empty()` + workdir status → `dirty`
- `branch.upstream()` + commit graph walk → `ahead` / `behind`
- `repo.head().peel_to_commit()` + walker → `last_commit`

### No Write Operations (Initial Scope)

Vobes reads Git state. It does not commit, push, branch, or merge.

## Caching

Git status is cached. Refreshed on:

- `vbs sync`
- manual refresh in desktop
- TTL expiry (configurable, default 60s)

This keeps `vbs list` instant even with many repos.

## Non-Goals (for now)

- No diffs.
- No blame.
- No branch creation.
- No merge UI.

These are not Vobes' job. We surface state. The user opens their editor
or terminal for actions.
