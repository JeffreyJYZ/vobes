<script lang="ts">
	// Generic dropdown — a button that opens a popover of options.
	// Replaces native `<select>` everywhere, because on macOS the OS
	// ignores most of the CSS we throw at the native control, and the
	// look ends up inconsistent with the rest of the app.

	import { clickOutside } from "../lib/clickOutside";

	type Option<V extends string = string> = { value: V; label: string };

	export let value: string;
	export let options: Option[];
	export let ariaLabel: string = "";
	export let onChange: (v: string) => void = () => {};
	export let width: string = "auto";
	/// When true, the trigger shows only a chevron — useful for a
	/// "split button" pattern where a sibling button carries the
	/// action label and this control just picks a target. The menu
	/// still lists labels so the user can see what they are choosing.
	export let compact: boolean = false;
	/// Anchor the popover to the right edge of the trigger instead of
	/// stretching to the trigger width. Needed when the trigger is a
	/// narrow chevron button next to a wide action button.
	export let menuAlign: "left" | "right" = "left";

	let open = false;
	let triggerEl: HTMLButtonElement;

	function pick(v: string) {
		onChange(v);
		open = false;
		triggerEl?.focus();
	}

	function onTriggerKey(e: KeyboardEvent) {
		if (e.key === "ArrowDown" || e.key === "Enter" || e.key === " ") {
			e.preventDefault();
			open = true;
		} else if (e.key === "Escape" && open) {
			e.preventDefault();
			open = false;
		}
	}

	function onMenuKey(e: KeyboardEvent) {
		if (e.key === "Escape") {
			e.preventDefault();
			open = false;
			triggerEl?.focus();
		}
	}

	$: current = options.find((o) => o.value === value);
</script>

<div
	class="select"
	class:compact
	style="width: {width}"
	use:clickOutside={() => (open = false)}
>
	<button
		type="button"
		class="trigger"
		class:open
		bind:this={triggerEl}
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-label={ariaLabel}
		title={compact ? (current?.label ?? value) : ""}
		on:click={() => (open = !open)}
		on:keydown={onTriggerKey}
	>
		{#if compact}
			<span class="chev" aria-hidden="true">▾</span>
		{:else}
			<span class="label">{current?.label ?? value}</span>
			<span class="chev" aria-hidden="true">▾</span>
		{/if}
	</button>
	{#if open}
		<ul
			class="menu"
			class:align-right={menuAlign === "right"}
			role="listbox"
			aria-label={ariaLabel}
			on:keydown={onMenuKey}
		>
			{#each options as o (o.value)}
				<li>
					<button
						type="button"
						class="opt"
						class:active={o.value === value}
						role="option"
						aria-selected={o.value === value}
						on:click={() => pick(o.value)}
					>
						{o.label}
						{#if o.value === value}<span
								class="check"
								aria-hidden="true">✓</span
							>{/if}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.select {
		position: relative;
		display: inline-block;
	}
	.select.compact {
		display: inline-flex;
	}
	.trigger {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
		padding: 7px 12px;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 8px;
		color: var(--fg);
		font: inherit;
		cursor: pointer;
		text-align: left;
		transition: border-color 0.12s ease;
	}
	.trigger:hover {
		border-color: var(--fg-faint);
	}
	.trigger.open {
		border-color: var(--accent);
		box-shadow: 0 0 0 2px var(--accent-soft);
	}
	.select.compact .trigger {
		width: auto;
		padding: 7px 8px;
		gap: 0;
		min-width: 28px;
		justify-content: center;
	}
	.trigger .label {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.trigger .chev {
		color: var(--fg-muted);
		font-size: 12px;
		font-weight: 700;
		transition: transform 0.15s ease;
	}
	.trigger.open .chev {
		transform: rotate(180deg);
	}
	.menu {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		z-index: 60;
		margin: 0;
		padding: 4px;
		list-style: none;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 8px;
		box-shadow: var(--shadow-md);
		min-width: 100%;
		animation: pop 0.1s ease;
	}
	.menu.align-right {
		left: auto;
		right: 0;
		min-width: 180px;
	}
	.menu li {
		margin: 0;
		padding: 0;
	}
	.opt {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 7px 10px;
		border: none;
		background: transparent;
		border-radius: 5px;
		color: var(--fg);
		font: inherit;
		font-size: 13px;
		cursor: pointer;
		text-align: left;
	}
	.opt:hover {
		background: var(--bg-sunken);
	}
	.opt.active {
		background: var(--accent-soft);
		color: var(--accent);
		font-weight: 600;
	}
	.opt .check {
		color: var(--accent);
		font-weight: 700;
	}
	@keyframes pop {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
	}
</style>
