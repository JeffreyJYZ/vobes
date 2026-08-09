<script lang="ts">
import {
	activity,
	allTags,
	attentionCount,
	onboardingDone,
	removeSavedFilter,
	savedFilters,
	searchQuery,
	view,
	vobes,
} from "../lib/stores"
import type { ViewId } from "../lib/types"

const items: { id: ViewId; label: string }[] = [
	{ id: "dashboard", label: "Dashboard" },
	{ id: "activity", label: "Activity" },
	{ id: "settings", label: "Settings" },
]

function setView(id: ViewId) {
	view.set(id)
}

function applyFilter(q: string) {
	view.set("dashboard")
	searchQuery.set(q)
}
</script>

<aside class="sidebar">
  <div class="brand">
    <svg class="logo" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M3 4 L12 20 L21 4"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <h1>Vobes</h1>
  </div>

  <nav class="nav">
    {#each items as it (it.id)}
      <button
        type="button"
        class="nav-item"
        class:active={$view === it.id}
        on:click={() => setView(it.id)}
      >
        <span>{it.label}</span>
        {#if it.id === "dashboard" && $attentionCount > 0}
          <span class="pill attention" title="{$attentionCount} need attention">
            {$attentionCount}
          </span>
        {:else if it.id === "activity"}
          <span class="pill">{$activity.length}</span>
        {/if}
      </button>
    {/each}
  </nav>

  {#if $allTags.length > 0}
    <div class="saved">
      <div class="saved-head">Workspaces</div>
      {#each $allTags as t (t)}
        <button
          class="tag-btn"
          type="button"
          on:click={() => applyFilter(`tag:${t}`)}
          title={`Scope dashboard to vobes tagged “${t}”`}
        >
          <span class="tag-hash">#</span>
          <span class="lbl">{t}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if $savedFilters.length > 0}
    <div class="saved">
      <div class="saved-head">Saved views</div>
      {#each $savedFilters as f (f.id)}
        <div class="saved-row">
          <button
            class="saved-btn"
            type="button"
            on:click={() => applyFilter(f.query)}
            title="Apply filter: {f.query}"
          >
            <span class="lbl">{f.label}</span>
            <span class="hint">↩</span>
          </button>
          <button
            class="saved-x"
            type="button"
            on:click={() => removeSavedFilter(f.id).catch(() => {})}
            aria-label="Remove saved view"
            title="Remove"
          >×</button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="footer">
    {#if !$onboardingDone && $vobes.length === 0}
      <button class="setup" type="button" on:click={() => view.set("settings")}>
        Finish setup →
      </button>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    border-right: 1px solid var(--border);
    padding: 22px 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--bg-elevated);
    min-height: 0;
  }
  .brand {
    display: flex; align-items: center; gap: 10px;
    margin: 0 0 18px; padding: 0 6px;
  }
  .brand .logo { width: 26px; height: 26px; color: var(--accent); flex: none; }
  .brand h1 { font-size: 17px; font-weight: 700; letter-spacing: -0.01em; margin: 0; }
  .nav { display: flex; flex-direction: column; gap: 2px; }
  .nav-item {
    display: flex; align-items: center; justify-content: space-between;
    width: 100%;
    text-align: left;
    padding: 9px 12px;
    border: none; border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--fg-muted);
    background: transparent;
    font-weight: 500;
    transition: all 0.14s ease;
  }
  .nav-item:hover { background: var(--bg-sunken); color: var(--fg); }
  .nav-item.active { background: var(--accent-soft); color: var(--accent); font-weight: 600; }
  .pill {
    font-size: 11px; font-weight: 600;
    color: var(--fg-muted);
    background: var(--bg-sunken);
    border-radius: 999px;
    padding: 1px 7px;
  }
  .pill.attention {
    color: #fff;
    background: var(--warn);
  }
  .saved {
    margin-top: 18px;
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }
  .saved-head {
    font-size: 11px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.04em;
    color: var(--fg-faint);
    margin: 0 6px 6px;
  }
  .saved-row {
    display: flex; align-items: center; gap: 2px;
  }
  .saved-btn {
    flex: 1;
    display: flex; align-items: center; justify-content: space-between;
    background: transparent; border: none;
    padding: 6px 8px;
    color: var(--fg);
    cursor: pointer;
    border-radius: 6px;
    text-align: left;
    font: inherit;
    font-size: 13px;
  }
  .saved-btn:hover { background: var(--bg-sunken); }
  .saved-btn .lbl { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .saved-btn .hint { color: var(--fg-faint); font-size: 11px; }
  .saved-x {
    background: transparent; border: none; cursor: pointer;
    color: var(--fg-faint); font-size: 15px;
    padding: 0 6px;
    line-height: 1;
    border-radius: 4px;
  }
  .saved-x:hover { color: var(--fg); background: var(--bg-sunken); }
  .tag-btn {
    display: flex; align-items: center; gap: 4px;
    width: 100%;
    background: transparent; border: none;
    padding: 5px 8px;
    color: var(--fg-muted);
    cursor: pointer;
    border-radius: 6px;
    text-align: left;
    font: inherit;
    font-size: 13px;
  }
  .tag-btn:hover { background: var(--bg-sunken); color: var(--fg); }
  .tag-btn .tag-hash { color: var(--fg-faint); }
  .footer { margin-top: auto; padding: 8px 4px 0; }
  .setup {
    width: 100%; padding: 9px 12px;
    background: var(--accent-soft); color: var(--accent);
    border: none; border-radius: var(--radius-sm);
    font-weight: 600; cursor: pointer;
  }
</style>
