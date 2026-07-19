# AI Integration

Eventually, AI agents should naturally fit into this workflow.

This phase is intentionally last. The architecture supports it; the
implementation comes after the foundation is solid.

## Surface

Vobes exposes its data in three agent-friendly ways.

### 1. CLI Commands

```bash
vbs context <name>     # dump full vobe as JSON
vbs list --json        # machine-readable list
vbs log --json         # machine-readable activity
vbs show <name> --json # machine-readable detail
```

Any AI agent can shell out to `vbs` and parse the JSON.

### 2. MCP Server

A `vobes-mcp` crate exposes Vobes data via the Model Context Protocol.
Speaks JSON-RPC 2.0 over stdio (no external MCP SDK — minimal deps).
Lives in the same monorepo, reuses `vobes-cli::app::App`.

Tools exposed:

- `vobes_list` — list vobes
- `vobes_show` — show one vobe (+ recent activity)
- `vobes_recent_activity` — recent events
- `vobes_search` — find by name substring
- `vobes_context` — full structured context for a vobe (record + activity + dir entries)

Run it:

```bash
cargo run -p vobes-mcp
```

### 3. Watch Stream

`vbs watch` emits activity events to stdout (or a Unix socket / named
pipe). Agents subscribe to live events.

```bash
vbs watch
```

Emits NDJSON: a `{"type":"ready"}` line on start, then one JSON object
per new activity event per second. Stop with Ctrl-C.

## Why It Fits Naturally

Because we have a shared core with clean types, the data is already
structured and serializable. Adding an MCP or watch surface is just
another consumer of the same traits.

## Initial Use Cases (deferred)

- Summarize recent activity for a vobe.
- Suggest what to work on next.
- Draft a commit message from staged diff (uses `git2` already loaded).
- Detect stale projects (no activity in N days).
- Generate project notes from README + recent commits.

## What We Are NOT Building

- We are not an AI. We surface data so agents can act on it.
- We are not a chat UI. Desktop integration comes later, deliberately.

## Non-Goals

- No LLM calls inside Vobes core. Vobes is a data layer for agents.
- No local model hosting. Out of scope.
- No fine-tuning. Out of scope.
