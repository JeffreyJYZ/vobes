<script lang="ts">
	import { onMount, tick } from "svelte";
	import { clickOutside } from "../lib/clickOutside";

	export let x = 0;
	export let y = 0;
	export let items: { label: string; onClick: () => void; danger?: boolean }[] = [];
	export let onClose: () => void = () => {};

	let menuEl: HTMLElement;
	let posX = x;
	let posY = y;

	// Keep the menu inside the viewport. Measure after layout, then
	// nudge if it would overflow right or bottom.
	async function place() {
		await tick();
		if (!menuEl) return;
		const r = menuEl.getBoundingClientRect();
		const vw = window.innerWidth;
		const vh = window.innerHeight;
		if (r.right > vw - 8) posX = Math.max(8, x - r.width);
		if (r.bottom > vh - 8) posY = Math.max(8, y - r.height);
	}

	onMount(place);

	$: x, y, place();
</script>

{#if items.length > 0}
	<div
		bind:this={menuEl}
		class="ctx"
		role="menu"
		style="left:{posX}px; top:{posY}px;"
		use:clickOutside={() => onClose()}
	>
		{#each items as it (it.label)}
			<button
				type="button"
				class="item"
				class:danger={it.danger}
				role="menuitem"
				on:click={() => {
					it.onClick();
					onClose();
				}}
			>
				{it.label}
			</button>
		{/each}
	</div>
{/if}

<style>
	.ctx {
		position: fixed;
		z-index: 100;
		min-width: 160px;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 4px;
		box-shadow: var(--shadow-md);
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.item {
		display: block;
		width: 100%;
		text-align: left;
		background: transparent;
		border: none;
		padding: 7px 10px;
		font: inherit;
		font-size: 13px;
		color: var(--fg);
		cursor: pointer;
		border-radius: 5px;
	}
	.item:hover {
		background: var(--bg-sunken);
	}
	.item.danger {
		color: var(--danger);
	}
	.item.danger:hover {
		background: color-mix(in srgb, var(--danger) 12%, transparent);
	}
</style>