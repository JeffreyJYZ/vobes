<script lang="ts">
import { relativeTime } from "../lib/format"
import { activity, kindLabel, refresh, scanning, vobes } from "../lib/stores"
import type { ActivityEvent, Vobe } from "../lib/types"

let kindFilter: string = "all"
let vobeFilter: string = "all"

$: filtered = filterEvents($activity, $vobes, kindFilter, vobeFilter)
$: grouped = groupByDay(filtered)

function filterEvents(
	all: ActivityEvent[],
	_vobesList: Vobe[],
	kind: string,
	vobeId: string,
): ActivityEvent[] {
	return all.filter((e) => {
		if (kind !== "all" && e.kind !== kind) return false
		if (vobeId !== "all" && e.vobe_id !== vobeId) return false
		return true
	})
}

function groupByDay(
	events: ActivityEvent[],
): { day: string; items: ActivityEvent[] }[] {
	const out = new Map<string, ActivityEvent[]>()
	for (const e of events) {
		const d = new Date(e.timestamp)
		const key = d.toLocaleDateString(undefined, {
			weekday: "long",
			month: "short",
			day: "numeric",
		})
		const arr = out.get(key) ?? []
		arr.push(e)
		out.set(key, arr)
	}
	return Array.from(out.entries()).map(([day, items]) => ({ day, items }))
}

function vobeName(id: string, all: Vobe[]): string {
	return all.find((v) => v.id === id)?.name ?? id.slice(0, 8)
}

$: kinds = Array.from(new Set($activity.map((e) => e.kind))).sort()
</script>

<div class="activity">
  <div class="head">
    <h2>Activity</h2>
    <button on:click={() => refresh()} disabled={$scanning}>
      {$scanning ? "Refreshing…" : "Refresh"}
    </button>
  </div>

  <div class="filters">
    <label>
      <span>Kind</span>
      <select bind:value={kindFilter}>
        <option value="all">All</option>
        {#each kinds as k (k)}
          <option value={k}>{kindLabel(k)}</option>
        {/each}
      </select>
    </label>
    <label>
      <span>Vobe</span>
      <select bind:value={vobeFilter}>
        <option value="all">All vobes</option>
        {#each $vobes as v (v.id)}
          <option value={v.id}>{v.name}</option>
        {/each}
      </select>
    </label>
    <span class="count">{$activity.length} events</span>
  </div>

  {#if filtered.length === 0}
    <div class="empty">
      <strong>Nothing to show</strong>
      <p>
        Activity appears here when you scan, open, or modify vobes. As you
        use Vobes, this fills up.
      </p>
    </div>
  {:else}
    {#each grouped as g (g.day)}
      <div class="day">
        <h3>{g.day}</h3>
        <ul>
          {#each g.items as e (e.id ?? `${e.vobe_id}-${e.timestamp}`)}
            <li>
              <span class="when">{relativeTime(e.timestamp)}</span>
              <span class="what">
                <span class="verb">{kindLabel(e.kind)}</span>
                <span class="vobe">{vobeName(e.vobe_id, $vobes)}</span>
                {#if e.detail}
                  <span class="detail">— {e.detail}</span>
                {/if}
              </span>
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  {/if}
</div>

<style>
  .activity { max-width: 800px; margin: 0 auto; }
  .head {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 18px;
  }
  .head h2 { margin: 0; }
  .filters {
    display: flex; align-items: center; gap: 14px;
    margin-bottom: 22px;
    padding: 10px 14px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .filters label {
    display: flex; align-items: center; gap: 8px;
    color: var(--fg-muted); font-size: 12.5px;
  }
  .filters select {
    padding: 5px 8px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--fg);
    font: inherit;
  }
  .count {
    color: var(--fg-faint); font-size: 12px;
    margin-left: auto;
  }
  .day { margin-bottom: 22px; }
  .day h3 {
    margin: 0 0 8px;
    font-size: 12px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.04em;
    color: var(--fg-faint);
  }
  .day ul {
    list-style: none; margin: 0; padding: 0;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
  }
  .day li {
    display: flex; gap: 12px; align-items: baseline;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    font-size: 13.5px;
  }
  .day li:last-child { border-bottom: none; }
  .when {
    color: var(--fg-faint); font-size: 12px;
    min-width: 70px;
  }
  .what { display: flex; gap: 6px; flex-wrap: wrap; }
  .verb { color: var(--accent); font-weight: 600; }
  .vobe { font-weight: 600; }
  .detail { color: var(--fg-muted); }
  .empty {
    color: var(--fg-muted); text-align: center;
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
</style>
