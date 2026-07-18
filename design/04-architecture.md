# Architecture Goals

## Core Principle

One **shared core**. Business logic must not be duplicated between
desktop and CLI.

## What Belongs in the Shared Core

- project management
- filesystem operations
- metadata
- scanning
- Git interactions
- configuration
- models
- error types
- storage

## What Stays Platform-Specific

- UI rendering (desktop)
- terminal formatting (CLI)
- platform paths
- OS integration (notifications, launchers)

## Architectural Rules

### Reusable Modules

Every package should have a single clear responsibility and be reusable
from any interface.

### Well-Defined Boundaries

A crate's public API should be small, stable, and obvious.

### Shared Domain Models

The same `Vobe`, `ActivityEvent`, `GitInfo` types flow through CLI,
desktop, and any future interface.

### Clean APIs

- Trait-based abstractions at package boundaries.
- Errors typed and `Result`-based, never `panic`.
- No hidden side effects in constructors.

### Future Extensibility

Custom metadata, tags, and notes must work without code changes.

### No Enterprise Complexity

- No DI containers.
- No abstract base class hierarchies.
- No "manager" layers around "service" layers.
- If an abstraction doesn't pay for itself, delete it.

### Scale Naturally

Today's two interfaces (CLI + desktop) should not require rewrites
when a third (MCP server, web UI) is added.

## Boundaries Diagram

```
        +-------------------+
        |  vobes-core       |  <- models, traits, errors
        +-------------------+
        ^    ^    ^    ^
        |    |    |    |
   +----+ +--+ +--+ +--+----+
   |      |  |  |  |       |
   |      |  |  |  |       |
+-----+ +-----+ +-----+ +--------+
|scan | | git | |store| |activity|
+-----+ +-----+ +-----+ +--------+
   ^      ^      ^        ^
   +------+------+--------+
                  |
        +---------+--------+
        | vobes-cli  /  vobes-desktop |
        +-----------------------------+
```

Platform code (CLI / Desktop) lives at the edges. Everything else is shared.
