<script lang="ts">
	import { relativeTime } from "../lib/format";
	import { attentionItems, selectedVobe, view, vobes } from "../lib/stores";
	import type { Vobe } from "../lib/types";

	// The panel shows the first six; the sidebar count uses the same
	// rule over the full list.
	$: all = attentionItems($vobes);
	$: items = all.slice(0, 6);

	function open(v: Vobe) {
		selectedVobe.set(v);
		view.set("projects");
	}
</script>

{#if items.length > 0}
	<section class="attention">
		<header>
			<h3>Needs attention</h3>
			<span class="count">{all.length}</span>
		</header>
		<ul>
			{#each items as it (it.vobe.id)}
				<li>
					<button class="item" on:click={() => open(it.vobe)}>
						<span class="bar" data-w={it.weight}></span>
						<span class="name">{it.vobe.name}</span>
						<span class="reason">{it.reason}</span>
						<span class="branch">{it.vobe.git?.branch ?? ""}</span>
						<span class="when"
							>{relativeTime(it.vobe.last_modified)}</span
						>
					</button>
				</li>
			{/each}
		</ul>
	</section>
{/if}

<style>
	.attention {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 12px;
		padding: 14px 18px;
		margin-bottom: 22px;
	}
	header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 10px;
	}
	header h3 {
		margin: 0;
		font-size: 13px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--fg-faint);
	}
	.count {
		font-size: 11px;
		font-weight: 700;
		color: var(--warn);
		background: color-mix(in srgb, var(--warn) 18%, transparent);
		border-radius: 999px;
		padding: 1px 7px;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.item {
		display: grid;
		grid-template-columns: 3px 1.5fr 1fr 1fr auto;
		align-items: center;
		gap: 12px;
		width: 100%;
		text-align: left;
		background: transparent;
		border: none;
		padding: 8px 6px;
		border-radius: 8px;
		color: var(--fg);
		cursor: pointer;
		font: inherit;
	}
	.item:hover {
		background: var(--bg-sunken);
	}
	.bar {
		width: 3px;
		height: 22px;
		border-radius: 2px;
		background: var(--warn);
	}
	.bar[data-w="1"] {
		background: var(--fg-faint);
	}
	.bar[data-w="2"] {
		background: var(--accent);
	}
	.bar[data-w="3"] {
		background: var(--warn);
	}
	.name {
		font-weight: 600;
	}
	.reason {
		color: var(--warn);
		font-size: 12.5px;
	}
	.branch {
		color: var(--fg-muted);
		font-size: 12.5px;
		font-family: ui-monospace, Menlo, monospace;
	}
	.when {
		color: var(--fg-faint);
		font-size: 12px;
	}
</style>
