# Vobes Improvement Roadmap

Rough phases + features to take Vobes from "demo" to "dev actually opens it
every day". Not file-specific, not code-specific. Each phase ships something
usable on its own.

## North Star

A dev opens Vobes in the morning, sees in 5 seconds **what needs attention**
(dirty repos, unpushed work, stale projects), and jumps into any project in
≤2 keystrokes. Mouse optional. Zero config required to get there.

## Current Pain (why it feels horrible)

- No search, no filter, no sort — fine at 3 projects, useless at 30.
- Mouse-only. Design doc says keyboard-first; app has zero shortcuts.
- Dashboard is a flat card grid — shows data, not insight. Nothing answers
  "what should I look at?"
- No onboarding: first launch shows an empty box that says "click Scan".
- Settings don't exist in-app — editing TOML by hand is the only way.
- Feedback is broken: success messages render as errors; no toasts.
- Git info is shallow (branch + dirty). No commits, no unpushed detail,
  no recent history.
- Data goes stale silently. Manual refresh only.
- Notes/TODOs promised in the README don't exist anywhere.
- Activity is a raw table dump. No filtering, no grouping, no meaning.
- One monolithic UI — no component structure to grow into.

---

## Phase 1 — Daily-Driver Foundations

Goal: a dev with many projects can find and open any of them fast, without
touching the mouse, and the app never feels broken.

Features:

- **Global fuzzy search** across project names, paths, tags. One keystroke
  to focus, always visible.
- **Keyboard-first navigation**: arrow keys through lists, Enter to open,
  number-key or letter shortcuts to switch views, `?` opens a shortcut
  cheatsheet, Esc clears/back.
- **Command palette** (Cmd/Ctrl+K): jump to project, run scan, open in
  editor, switch view — every action reachable by typing.
- **Sorting & grouping**: by recency, name, language; "dirty first" option.
  Remember the choice.
- **Pins / favorites**: pinned projects float to top everywhere.
- **Proper feedback system**: toasts for success/error/info; inline spinners
  only where data is actually loading; never block the whole UI.
- **Guided first-run**: detect empty store → walk through picking scan roots
  → run first scan with live progress → land on populated dashboard.
- **Settings view in-app**: scan roots, editor command, theme, refresh
  interval. Writes the same config file the CLI uses. TOML hand-editing
  stays possible but never required.
- **Componentized UI shell**: sidebar, list, detail, palette, toast as
  separate pieces so later phases are additive.

Exit criteria: open app, type 3 letters, hit Enter, project opens in editor.
No mouse. No config file touched.

---

## Phase 2 — Signal, Not Noise

Goal: dashboard answers "what needs my attention?" instead of "here is
everything you have."

Features:

- **Attention section** at top of dashboard: dirty repos, unpushed commits
  (ahead > 0), behind upstream, projects not touched in N days. Each item
  is one click to act on.
- **Health/status badges with meaning**: clean + pushed = quiet; dirty,
  ahead, behind, stale = visible but calm. Color used sparingly, per the
  design ethos.
- **Rich project cards**: last commit message + time, branch, language
  badge, ahead/behind counts. Density on demand — compact list view vs
  card view toggle.
- **Recent activity digest** on dashboard: last ~5 events grouped by day,
  human phrasing ("opened api-server", "scanned 12 projects").
- **Auto-refresh**: git status refreshes on window focus + on a sensible
  interval. Manual refresh still available. Never a spinner for cached
  data — show stale instantly, update in place.
- **Stale-data indicators**: "updated 4m ago" timestamps so numbers are
  trustworthy.
- **Hide/archive projects**: keep the default view to active work without
  deleting records.

Exit criteria: morning check = open app, glance at attention section, done
in under 10 seconds.

---

## Phase 3 — Project Depth

Goal: clicking a project gives a dev everything they'd otherwise open a
terminal to check.

Features:

- **Rich detail view**: recent commits (message, author, time), full git
  status summary (staged/unstaged/untracked counts), remotes, last fetch
  time.
- **One-click actions per project**: open in editor, open in terminal,
  reveal in file manager, copy path, open remote (GitHub/GitLab) in
  browser.
- **Per-project activity timeline**: everything Vobes knows about this
  project, grouped by day.
- **Notes per project**: freeform markdown scratchpad, stored locally.
  First place a dev dumps "where was I?" context.
- **TODO scraping**: surface `TODO`/`FIXME` comments (and/or a project
  todo file) as a checklist on the detail view.
- **Tag editing in-app**: add/remove tags, filter list by tag.
- **README peek**: render the project's README inline for instant context.

Exit criteria: dev checks git state and picks up where they left off
without opening a terminal.

---

## Phase 4 — Always-On & System Integration

Goal: Vobes earns a permanent spot in the workflow — lives in the
background, surfaces info where the dev already is.

Features:

- **Menu bar / system tray presence**: attention count at a glance, quick
  project switcher from the tray, launch on login option.
- **Global hotkey**: summon the quick-switcher from anywhere, type a few
  letters, Enter → project opens in editor. The killer feature.
- **Background sync daemon-lite**: app refreshes git state on a schedule
  even when the window is closed (feeds tray badge).
- **File-watching**: watched projects update near-instantly instead of on
  interval.
- **Deep links / URL scheme**: `vobes://open/<name>` so scripts, Alfred,
  Raycast, and agents can drive the app.
- **Native polish**: real dark/light theme follow-system, proper window
  chrome per OS, sensible default window size/position memory.
- **Notifications (opt-in, calm)**: only for things that matter — e.g.
  "main is 20 behind upstream". Off by default.

Exit criteria: dev switches projects via global hotkey without ever opening
the main window.

---

## Phase 5 — Agent-Native & Power Surface

Goal: the MCP/CLI strengths become visible product features, and power
users can extend everything.

Features:

- **Agent activity feed**: distinguish events recorded by AI agents vs
  human actions; show "what my agent touched today" as a first-class view.
- **Context-pack UI**: one click to copy a project's full context pack
  (the `vbs context` payload) to clipboard for pasting into an agent.
- **Saved filters / views**: "dirty rust projects", "work repos behind
  upstream" — pin as sidebar sections.
- **Workspaces / groups**: tag sets of projects as a workspace; switch the
  whole dashboard scope with one shortcut.
- **Export & backup UX**: scheduled JSON snapshots, visible snapshot
  history, one-click restore point listing.
- **CLI ↔ desktop parity**: anything the desktop can do, `vbs` can do,
  and vice versa. Document the matrix.
- **Plugin hooks (design only at this phase)**: define where custom
  detectors, actions, and dashboard widgets would plug in.

Exit criteria: an AI agent + a human can share Vobes as the common picture
of "all my projects", each seeing what the other did.

---

## Non-Goals (stay honest to the ethos)

- No accounts, no cloud sync, no telemetry.
- No becoming a git GUI (no staging/committing UI — link out to the real
  tools).
- No notification spam, no badges-for-badges-sake, no gamification.
- No feature that requires configuration before it works.

## Sizing Reality Check

| Phase | Theme | Feels like |
|---|---|---|
| 1 | Foundations | 2–3 focused weeks solo |
| 2 | Signal | 2 weeks |
| 3 | Depth | 2–3 weeks |
| 4 | Always-on | 3–4 weeks (tray/hotkey/watchers are platform-fiddly) |
| 5 | Agent-native | ongoing, pick per release |

Ship order matters more than completeness: Phase 1 alone already makes the
app usable; everything after compounds.
