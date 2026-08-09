<script lang="ts">
import Attention from "../components/Attention.svelte"
import Select from "../components/Select.svelte"
import VobeCard from "../components/VobeCard.svelte"
import { relativeTime } from "../lib/format"
import {
	addSavedFilter,
	density,
	doScan,
	lastRefreshed,
	pushToast,
	scanning,
	searchQuery,
	selectedVobe,
	sortKey,
	view,
	vobes,
} from "../lib/stores"
import type { SortKey, Vobe } from "../lib/types"

let searchEl: HTMLInputElement

$: filtered = filterAndSort($vobes, $searchQuery, $sortKey)

function filterAndSort(all: Vobe[], q: string, key: SortKey): Vobe[] {
	const ql = q.trim().toLowerCase()
	let arr = all
	if (ql) {
		arr = arr.filter(
			(v) =>
				v.name.toLowerCase().includes(ql) ||
				(v.path ?? "").toLowerCase().includes(ql) ||
				v.tags.some((t) => t.toLowerCase().includes(ql)) ||
				(v.language ?? "").toLowerCase().includes(ql) ||
				(v.framework ?? "").toLowerCase().includes(ql),
		)
	}
	return [...arr].sort((a, b) => {
		if (a.pinned !== b.pinned) return a.pinned ? -1 : 1
		switch (key) {
			case "name":
				return a.name.localeCompare(b.name)
			case "created_at":
				return (b.created_at ?? "").localeCompare(a.created_at ?? "")
			case "last_opened":
				return (b.last_opened ?? "").localeCompare(a.last_opened ?? "")
			default:
				return (b.last_modified ?? "").localeCompare(
					a.last_modified ?? "",
				)
		}
	})
}

function open(v: Vobe) {
	selectedVobe.set(v)
	view.set("projects")
}

function setSort(k: SortKey) {
	sortKey.set(k)
}

function onSortPick(v: string) {
	setSort(v as SortKey)
}

const sortOptions = [
	{ value: "last_modified", label: "Recently modified" },
	{ value: "last_opened", label: "Recently opened" },
	{ value: "name", label: "Name" },
	{ value: "created_at", label: "Newest" },
]

function _focusSearch() {
	searchEl?.focus()
}
</script>

<div class="dashboard">
  <Attention />

  <div class="toolbar">
    <div class="left">
      <h2>Dashboard</h2>
      <span class="muted">
        {$vobes.length} vobe{$vobes.length === 1 ? "" : "s"} ·
        updated {relativeTime(toISOStr($lastRefreshed))}
      </span>
    </div>
    <div class="right">
      <div class="search">
        <span class="hint">/</span>
        <input
          bind:this={searchEl}
          bind:value={$searchQuery}
          type="text"
          placeholder="Search vobes…"
          aria-label="Search vobes"
        />
      </div>
      <Select
        ariaLabel="Sort by"
        value={$sortKey}
        options={sortOptions}
        onChange={onSortPick}
        width="180px"
      />
      <button
        class="density"
        on:click={() => density.update((d) => (d === "comfy" ? "compact" : "comfy"))}
        title="Toggle density"
      >
        {$density === "comfy" ? "▦" : "☰"}
      </button>
      <button class="primary" on:click={doScan} disabled={$scanning}>
        {$scanning ? "Scanning…" : "Scan"}
      </button>
      <button
        on:click={async () => {
          const q = $searchQuery.trim();
          if (!q) {
            pushToast({ kind: "info", message: "Type a search first." });
            return;
          }
          try {
            await addSavedFilter(`View: ${q}`, q);
            pushToast({ kind: "success", message: `Saved view “${q}”.` });
          } catch (_e) {
            pushToast({ kind: "error", message: "Could not save view." });
          }
        }}
        disabled={!$searchQuery.trim()}
        title="Pin this search to the sidebar"
      >
        Save view
      </button>
    </div>
  </div>

  {#if $vobes.length === 0}
    <div class="empty">
      <strong>No vobes yet</strong>
      <p>
        Configure scan roots in <button class="link" on:click={() => view.set("settings")}>Settings</button>,
        then hit <kbd>Scan</kbd>.
      </p>
    </div>
  {:else if filtered.length === 0}
    <div class="empty">
      <strong>No matches</strong>
      <p>Try a different search or <button class="link" on:click={() => searchQuery.set("")}>clear it</button>.</p>
    </div>
  {:else}
    <div class="grid" class:list={$density === "compact"}>
      {#each filtered as v (v.id)}
        <button class="card-btn" on:click={() => open(v)}>
          <VobeCard vobe={v} compact={$density === "compact"} />
        </button>
      {/each}
    </div>
  {/if}
</div>

<script context="module" lang="ts">
  // Tiny helper for the timestamp in the toolbar.
  function toISOStr(ms: number): string | null {
    if (!ms) return null;
    return new Date(ms).toISOString();
  }
</script>

<style>
  .dashboard { max-width: 1100px; margin: 0 auto; }
  .toolbar {
    display: flex; align-items: center; justify-content: space-between;
    flex-wrap: wrap;
    margin-bottom: 18px; gap: 12px;
  }
  .left { min-width: 0; flex: 1 1 200px; }
  .left h2 { margin: 0 0 2px; }
  .muted { color: var(--fg-muted); font-size: 12.5px; }
  .right {
    display: flex; align-items: center; gap: 8px;
    flex-wrap: nowrap;
  }
  .right > * { flex-shrink: 0; white-space: nowrap; }
  .search {
    position: relative;
    display: flex; align-items: center;
    flex: 1 1 100px;
    min-width: 0;
  }
  .search input {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 6px 10px 6px 38px;
    color: var(--fg);
    font: inherit;
    width: 100%;
    min-width: 0;
  }
  .search input:focus {
    outline: 2px solid var(--accent-soft); border-color: var(--accent);
  }
  .search .hint {
    position: absolute; left: 10px;
    color: var(--fg-faint);
    font-size: 11px;
    font-family: ui-monospace, Menlo, monospace;
    background: var(--bg-sunken);
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 5px 6px;
    line-height: 1;
  }
  .density {
    width: 32px; height: 32px;
    padding: 0; display: grid; place-items: center;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--fg-muted);
    cursor: pointer;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 14px;
  }
  .grid.list {
    grid-template-columns: 1fr;
    gap: 6px;
  }
  .card-btn {
    background: transparent; border: none; padding: 0;
    text-align: left; cursor: pointer; color: inherit;
    display: block;
  }
  .card-btn :global(.card) { transform: none; }
  .empty {
    color: var(--fg-muted);
    text-align: center;
    padding: 64px 24px;
    border: 1px dashed var(--border);
    border-radius: 12px;
    background: var(--bg-elevated);
  }
  .empty strong {
    display: block; color: var(--fg);
    font-size: 15px; margin-bottom: 6px;
  }
  .empty p { margin: 0; font-size: 13.5px; }
  .link {
    background: transparent; border: none; padding: 0;
    color: var(--accent); cursor: pointer; text-decoration: underline;
    font: inherit;
  }
  kbd {
    font-family: ui-monospace, Menlo, monospace;
    background: var(--bg-sunken);
    color: var(--fg-muted);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0 5px;
    font-size: 11.5px;
  }
</style>
