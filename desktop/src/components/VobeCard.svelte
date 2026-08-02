<script lang="ts">
	import {
		frameworkLabel,
		languageLabel,
		packageManagerLabel,
		relativeTime,
	} from "../lib/format";
	import type { Vobe } from "../lib/types";

	export let vobe: Vobe;
	export let compact = false;

	$: git = vobe.git;
	$: needsAttention = !!git && (git.dirty || git.ahead > 0 || git.behind > 0);
	$: pinned = vobe.pinned;
	$: lang = languageLabel(vobe.language);
	$: fw = frameworkLabel(vobe.framework);
	$: pm = packageManagerLabel(vobe.package_manager);
	$: isVanilla = !vobe.language && !vobe.framework;
</script>

<div class="card" class:compact class:attention={needsAttention}>
	<div class="row1">
		<span class="name">{vobe.name}</span>
		{#if pinned}
			<span class="pin" title="Pinned">★</span>
		{/if}
		<span class="lang" class:vanilla={isVanilla}>{lang}</span>
	</div>
	{#if !compact}
		<div class="meta">
			<span class:vanilla={fw === "Vanilla"}>{fw}</span>
			<span>·</span>
			<span class:vanilla={pm === "None"}>{pm}</span>
		</div>
	{/if}
	<div class="badges">
		{#if git}
			<span class="badge" title="Branch">{git.branch}</span>
			{#if git.dirty}
				<span class="badge dirty" title="Uncommitted changes"
					>dirty</span
				>
			{/if}
			{#if git.ahead > 0}
				<span class="badge ahead" title="Commits to push"
					>↑{git.ahead}</span
				>
			{/if}
			{#if git.behind > 0}
				<span class="badge behind" title="Commits to pull"
					>↓{git.behind}</span
				>
			{/if}
		{/if}
		<span class="badge ghost">{relativeTime(vobe.last_modified)}</span>
	</div>
	<div class="path" title={vobe.path}>{vobe.path}</div>
</div>

<style>
	.card {
		display: block;
		text-align: left;
		border: 1px solid var(--border);
		border-radius: 12px;
		padding: 14px 16px;
		background: var(--bg-elevated);
		cursor: pointer;
		box-shadow: var(--shadow-sm);
		transition: all 0.14s ease;
	}
	.card:hover {
		border-color: var(--accent);
		box-shadow: var(--shadow-md);
		transform: translateY(-1px);
	}
	.card.attention {
		border-color: color-mix(in srgb, var(--warn) 40%, var(--border));
	}
	.card.compact {
		padding: 10px 12px;
	}
	.row1 {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.name {
		font-weight: 650;
		font-size: 15px;
		letter-spacing: -0.01em;
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.lang {
		font-size: 11px;
		font-weight: 600;
		color: var(--fg-muted);
		background: var(--bg-sunken);
		border-radius: 4px;
		padding: 1px 6px;
	}
	.lang.vanilla,
	.vanilla {
		color: var(--fg-faint);
		font-weight: 500;
	}
	.meta .vanilla {
		background: transparent;
	}
	.pin {
		color: var(--accent);
		font-size: 13px;
	}
	.meta {
		color: var(--fg-muted);
		font-size: 12.5px;
		display: flex;
		gap: 6px;
		margin: 4px 0 8px;
	}
	.badges {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		font-size: 11.5px;
	}
	.badge {
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--bg-sunken);
		color: var(--fg-muted);
		font-weight: 500;
	}
	.badge.dirty {
		background: color-mix(in srgb, var(--warn) 18%, transparent);
		color: var(--warn);
	}
	.badge.ahead {
		background: color-mix(in srgb, var(--accent) 16%, transparent);
		color: var(--accent);
	}
	.badge.behind {
		background: color-mix(in srgb, var(--danger) 16%, transparent);
		color: var(--danger);
	}
	.badge.ghost {
		background: transparent;
	}
	.path {
		color: var(--fg-faint);
		font-size: 11.5px;
		margin-top: 10px;
		font-family: ui-monospace, Menlo, monospace;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card.compact .meta {
		display: none;
	}
	.card.compact .path {
		display: none;
	}
	.card.compact .row1 {
		margin-bottom: 4px;
	}
</style>
