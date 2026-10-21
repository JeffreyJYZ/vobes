<script lang="ts">
	import { onMount } from "svelte";
	import {
		dismissedVersion,
		dismissOffer,
		installNow,
		LATEST_RELEASE_URL,
		startUpdateWatcher,
		updates,
	} from "../lib/updates";

	// Checks on mount, then periodically and on focus, so a window left open
	// still learns about a release. Silent by design: offline, rate-limited or
	// plugin-less builds just stay quiet, and the app is unaffected.
	onMount(() => startUpdateWatcher());

	$: offer = $updates.offer;
	$: visible =
		offer !== null &&
		$updates.phase !== "idle" &&
		offer.version !== dismissedVersion();
</script>

{#if visible && offer}
	<div class="strip">
		<p>
			Version
			<a href={LATEST_RELEASE_URL} target="_blank" rel="noreferrer noopener"
				>{offer.version}</a
			>
			is available.
			{#if $updates.phase === "downloading"}
				<span class="muted">
					Downloading{$updates.percent === null
						? "…"
						: ` ${$updates.percent}%`}
				</span>
			{:else if $updates.phase === "failed" && $updates.message}
				<span class="muted">
					Update failed ({$updates.message}) — install it from the
					release page instead.
				</span>
			{/if}
		</p>
		{#if offer.kind === "updater" && $updates.phase !== "failed"}
			<button
				class="action"
				type="button"
				on:click={() => installNow()}
				disabled={$updates.phase === "downloading"}
			>
				{$updates.phase === "downloading" ? "Updating…" : "Update now"}
			</button>
		{:else}
			<a class="action" href={LATEST_RELEASE_URL} target="_blank" rel="noreferrer noopener">
				Release page
			</a>
		{/if}
		{#if $updates.phase !== "downloading"}
			<button
				class="dismiss"
				type="button"
				on:click={dismissOffer}
				aria-label="Dismiss update notice"
				title="Dismiss">×</button
			>
		{/if}
	</div>
{/if}

<style>
	.strip {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 14px;
		font-size: 12px;
		color: var(--fg-muted);
		background: var(--accent-soft);
		border-bottom: 1px solid var(--border);
	}
	.strip p {
		margin: 0;
		flex: 1;
		min-width: 0;
	}
	.strip a {
		color: var(--accent);
	}
	.muted {
		color: var(--fg-faint);
	}
	.action {
		flex: none;
		background: transparent;
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 3px 9px;
		font: inherit;
		font-weight: 600;
		color: var(--accent);
		cursor: pointer;
		text-decoration: none;
	}
	.action:disabled {
		color: var(--fg-faint);
		cursor: default;
	}
	.dismiss {
		flex: none;
		background: transparent;
		border: none;
		padding: 0 4px;
		font: inherit;
		font-size: 15px;
		line-height: 1;
		color: var(--fg-faint);
		cursor: pointer;
		border-radius: 4px;
	}
	.dismiss:hover {
		color: var(--fg);
		background: var(--bg-sunken);
	}
</style>
