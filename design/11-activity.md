# Activity

The system should understand developer activity.

## Goal

Answer questions like:

- What did I work on yesterday?
- Which project did I last open?
- What has been quiet for two weeks?
- Show me my activity this week.

## Initial Scope (Lightweight)

Vobes starts simple. Activity tracking is **event-based** and
**append-only**.

### Tracked Event Kinds

```rust
enum ActivityKind {
    Opened,      // user opened the project
    Modified,    // filesystem change detected
    Committed,   // git commit recorded
    Scanned,     // vobes scan picked up the project
    Created,     // first time tracked
    Closed,      // explicit close (future)
    Tagged,      // user added/changed tags
    Noted,       // user edited notes
}
```

### Event Shape

```rust
struct ActivityEvent {
    id: u64,
    vobe_id: VobeId,
    kind: ActivityKind,
    timestamp: DateTime<Utc>,
    detail: Option<String>, // free-form context
}
```

## Queries

| Query | Use |
|---|---|
| `recent(n)` | last N events globally |
| `for_vobe(id, n)` | last N events for one vobe |
| `in_range(from, to)` | events in time window |
| `by_kind(kind, n)` | filter by event type |

## Recording

- `vbs open <name>` records `Opened`.
- `vbs sync` records `Modified` when mtime changes.
- Git hook integration (future) records `Committed`.
- Scanner records `Scanned` / `Created` for new projects.

## Storage

Persisted in SQLite. See [Storage](./24-storage.md).

Lightweight initially: no aggregation, no analytics. Just a timeline.

## Future

- Time-spent tracking.
- Focus sessions.
- Heatmaps.
- AI summaries of activity.
