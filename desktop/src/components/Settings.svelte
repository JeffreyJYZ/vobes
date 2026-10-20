<script lang="ts">
	import { ask } from "@tauri-apps/plugin-dialog";
	import { onDestroy, onMount } from "svelte";
	import * as actions from "../lib/actions";
	import * as api from "../lib/api";
	import {
		applyConfig,
		config,
		configPath,
		configPaths,
		errorString,
		loadConfig,
		pushToast,
		refresh,
		settingsDirty,
		view,
	} from "../lib/stores";
	import {
		deleteSnapshot,
		exportJson,
		listSnapshots,
		restoreSnapshot,
	} from "../lib/api";
	import {
		LATEST_RELEASE_URL,
		checkForUpdates,
		installNow,
		updates,
	} from "../lib/updates";
	import type { Config, SnapshotInfo } from "../lib/types";

	let draft: Config | null = null;
	let dirty = false;
	let saving = false;
	let newExclude = "";

	let snapshots: SnapshotInfo[] = [];
	let snapshotBusy = false;

	onMount(async () => {
		if (!$config) await loadConfig();
		draft = $config ? structuredClone($config) : null;
		await loadSnapshots();
	});

	onDestroy(() => settingsDirty.set(false));

	async function loadSnapshots() {
		try {
			snapshots = await listSnapshots();
		} catch (e) {
			console.warn("listSnapshots failed:", e);
		}
	}

	async function exportNow() {
		snapshotBusy = true;
		try {
			const p = await exportJson();
			pushToast({
				kind: "success",
				message: `Exported snapshot to ${p}.`,
			});
			await loadSnapshots();
		} catch (e) {
			pushToast({
				kind: "error",
				message: `Export failed: ${errorString(e)}`,
			});
		} finally {
			snapshotBusy = false;
		}
	}

async function restore(s: SnapshotInfo) {
	const ok = await ask(
		`Restore ${s.name}? This replaces all current vobes and activity with the snapshot contents.`,
		{ title: "Vobes", kind: "warning" },
	);
	if (!ok) {
		return;
	}
		snapshotBusy = true;
		try {
			await restoreSnapshot(s.path);
			await refresh();
			pushToast({ kind: "success", message: `Restored ${s.name}.` });
		} catch (e) {
			pushToast({
				kind: "error",
				message: `Restore failed: ${errorString(e)}`,
			});
		} finally {
			snapshotBusy = false;
		}
	}

	async function remove(s: SnapshotInfo) {
	const ok = await ask(`Delete ${s.name}? This cannot be undone.`, {
		title: "Vobes",
		kind: "warning",
	});
	if (!ok) return;
	try {
			await deleteSnapshot(s.path);
			snapshots = snapshots.filter((x) => x.path !== s.path);
		} catch (e) {
			pushToast({
				kind: "error",
				message: `Delete failed: ${errorString(e)}`,
			});
		}
	}

	function fmtBytes(n: number): string {
		if (n < 1024) return `${n} B`;
		if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
		return `${(n / 1024 / 1024).toFixed(2)} MB`;
	}

	$: if (draft && $config) {
		dirty = JSON.stringify(draft) !== JSON.stringify($config);
		settingsDirty.set(dirty);
	}

	async function pickRoot() {
		const p = await actions.pickDir();
		if (p && draft && !draft.scan.roots.includes(p)) {
			draft.scan.roots = [...draft.scan.roots, p];
		}
	}

	function removeRoot(r: string) {
		if (!draft) return;
		draft.scan.roots = draft.scan.roots.filter((x) => x !== r);
	}

	function addExclude() {
		if (!draft) return;
		const v = newExclude.trim();
		if (v && !draft.scan.exclude.includes(v)) {
			draft.scan.exclude = [...draft.scan.exclude, v];
		}
		newExclude = "";
	}

	function removeExclude(x: string) {
		if (!draft) return;
		draft.scan.exclude = draft.scan.exclude.filter((t) => t !== x);
	}

	async function save() {
		if (!draft) return;
		saving = true;
		try {
			await applyConfig(draft);
			pushToast({ kind: "success", message: "Settings saved." });
		} finally {
			saving = false;
		}
	}

	// Leaving the view must not discard edits silently — the view guard
	// in stores asks first (settingsDirty), and the Save bar is sticky so
	// it is always in reach.
	async function done() {
		view.set("dashboard");
	}

	function onRootPathKeydown(e: KeyboardEvent) {
		if (e.key !== "Enter") return;
		if (!draft) return;
		const t = e.currentTarget as HTMLInputElement;
		if (t.value && !draft.scan.roots.includes(t.value)) {
			draft.scan.roots = [...draft.scan.roots, t.value];
		}
		t.value = "";
	}

	function onExcludeKeydown(e: KeyboardEvent) {
		if (e.key !== "Enter") return;
		addExclude();
	}
</script>

<div class="settings">
	<div class="head">
		<h2>Settings</h2>
		<div class="row">
			<button class="primary" on:click={save} disabled={!dirty || saving}>
				{saving ? "Saving…" : "Save changes"}
			</button>
		</div>
	</div>

	{#if !draft}
		<div class="muted">Loading…</div>
	{:else}
		<section class="block">
			<h3>Scan roots</h3>
			<p class="muted">
				Folders Vobes walks to discover projects. Use <code>~</code> for
				your home directory.
			</p>
			<div class="list">
				{#each draft.scan.roots as r (r)}
					<div class="row-tag">
						<code>{r}</code>
						<button
							class="x"
							on:click={() => removeRoot(r)}
							aria-label="Remove">×</button
						>
					</div>
				{/each}
				{#if draft.scan.roots.length === 0}
					<div class="muted small">
						No roots — Vobes will find nothing.
					</div>
				{/if}
			</div>
			<div class="row gap">
				<button on:click={pickRoot}>Pick folder…</button>
				<input
					type="text"
					placeholder="or paste a path"
					on:keydown={onRootPathKeydown}
				/>
			</div>
		</section>

		<section class="block">
			<h3>Scan options</h3>
			<div class="grid">
				<label>
					<span>Max depth</span>
					<input
						type="number"
						min="1"
						max="10"
						bind:value={draft.scan.max_depth}
					/>
				</label>
				<label class="check">
					<input
						type="checkbox"
						bind:checked={draft.scan.follow_symlinks}
					/>
					Follow symlinks
				</label>
			</div>
		</section>

		<section class="block">
			<h3>Excludes</h3>
			<p class="muted">
				Folder names to always skip (added on top of the built-in
				defaults).
			</p>
			<div class="list">
				{#each draft.scan.exclude as x (x)}
					<div class="row-tag">
						<span>{x}</span>
						<button
							class="x"
							on:click={() => removeExclude(x)}
							aria-label="Remove">×</button
						>
					</div>
				{/each}
			</div>
			<div class="row gap">
				<input
					type="text"
					bind:value={newExclude}
					placeholder="e.g. node_modules, .venv, dist"
					on:keydown={onExcludeKeydown}
				/>
				<button on:click={addExclude}>Add</button>
			</div>
		</section>

		<section class="block">
			<h3>Display</h3>
			<div class="grid">
				<label>
					<span>Theme</span>
					<select bind:value={draft.display.theme}>
						<option value="auto">Follow system</option>
						<option value="light">Light</option>
						<option value="dark">Dark</option>
					</select>
				</label>
			</div>
		</section>

		<section class="block">
			<h3>Updates</h3>
			<p class="muted small">
				Vobes checks for a newer release on launch and verifies its
				signature before installing.
			</p>
			<div class="row gap updater-row">
				<span class="mono small">
					{$updates.running ?? "—"}
					{#if $updates.phase === "checking"}
						· checking…
					{:else if $updates.phase === "current"}
						· up to date
					{:else if $updates.phase === "available"}
						· {$updates.offer?.version} available
					{:else if $updates.phase === "downloading"}
						· downloading{$updates.percent === null
							? "…"
							: ` ${$updates.percent}%`}
					{:else if $updates.phase === "failed"}
						· {$updates.message ?? "check failed"}
					{/if}
				</span>
				{#if $updates.phase === "available" && $updates.offer?.kind === "updater"}
					<button on:click={() => installNow()} disabled={saving}>
						Install {$updates.offer.version}
					</button>
				{:else if $updates.phase === "available" && $updates.offer}
					<a
						class="button-like"
						href={LATEST_RELEASE_URL}
						target="_blank"
						rel="noreferrer noopener">Release page</a
					>
				{:else}
					<button
						on:click={() => checkForUpdates()}
						disabled={$updates.phase === "checking" ||
							$updates.phase === "downloading"}
					>
						Check now
					</button>
				{/if}
			</div>
		</section>

		<section class="block">
			<h3>Data locations</h3>
			<p class="muted small">
				Read-only.
				{#if $configPaths.state_dir.includes("-dev")}
					Dev build — uses a separate directory so it can't overwrite
					a release install.
				{/if}
			</p>
			<div class="kv-paths">
				<div class="k">Config file</div>
				<div class="v mono">{$configPath || "—"}</div>
				<div class="k">State directory</div>
				<div class="v mono">{$configPaths.state_dir || "—"}</div>
				<div class="k">Database</div>
				<div class="v mono">{$configPaths.db || "—"}</div>
				<div class="k">Snapshots</div>
				<div class="v mono">{$configPaths.snapshots || "—"}</div>
			</div>
		</section>

		<section class="block">
			<div class="snap-head">
				<h3>Snapshots</h3>
				<button on:click={exportNow} disabled={snapshotBusy}>
					{snapshotBusy ? "Working…" : "Export now"}
				</button>
			</div>
			<p class="muted small">
				Snapshots are full backups of vobes + activity, written as
				timestamped JSON files in the snapshots directory. Restoring one
				replaces the current store.
			</p>
			{#if snapshots.length === 0}
				<div class="muted small">
					No snapshots yet — click “Export now” to create one.
				</div>
			{:else}
				<ul class="snap-list">
					{#each snapshots as s (s.path)}
						<li>
							<div class="snap-meta">
								<code>{s.name}</code>
								<span class="muted small"
									>{new Date(s.modified_at).toLocaleString()} ·
									{fmtBytes(s.size_bytes)}</span
								>
							</div>
							<div class="snap-actions">
								<button
									on:click={() => restore(s)}
									disabled={snapshotBusy}>Restore</button
								>
								<button
									class="x"
									on:click={() => remove(s)}
									aria-label="Delete snapshot">×</button
								>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		</section>

		<section class="block">
			<h3>Desktop</h3>
			<p class="muted small">Desktop only. The CLI ignores these.</p>
			<div class="grid">
				<label class="check">
					<input
						type="checkbox"
						bind:checked={draft.desktop.notify_behind}
					/>
					Notify when a vobe is behind upstream
				</label>
			</div>
		</section>

		<section class="block danger-zone">
			<h3>Danger zone</h3>
			<p class="muted small">
				Both actions are irreversible. Reset re-scans from your roots;
				Factory Reset wipes the database, config, and snapshots —
				Vobes restarts as if freshly installed.
			</p>
			<div class="row gap">
				<button
					class="danger"
					on:click={async () => {
						const ok = await ask(
							"Removes every vobe from Vobes (your files on disk are untouched) and rediscovers them from your scan roots. Activity history is also cleared.",
							{ title: "Reset tracked vobes", kind: "warning" },
						);
						if (!ok) return;
						try {
							const n = await api.resetAndRescan();
							await refresh({ silent: true });
							pushToast({ kind: "success", message: `Reset complete: ${n} vobe${n === 1 ? "" : "s"} re-discovered.` });
						} catch (e) {
							pushToast({ kind: "error", message: errorString(e) });
						}
					}}
				>
					Reset tracked vobes
				</button>
				<button
					class="danger"
					on:click={async () => {
						const ok = await ask(
							"Deletes the database, config.toml, and all snapshots. Vobes restarts as if you just installed it.",
							{ title: "Factory reset Vobes", kind: "warning" },
						);
						if (!ok) return;
						try {
							await api.factoryReset();
							pushToast({ kind: "info", message: "Factory reset complete. Restart Vobes to set up again." });
						} catch (e) {
							pushToast({ kind: "error", message: errorString(e) });
						}
					}}
				>
					Factory reset Vobes
				</button>
			</div>
		</section>

		<div class="actions">
			{#if dirty}
				<span class="unsaved">Unsaved changes</span>
			{/if}
			<button on:click={done}>Done</button>
			<button class="primary" on:click={save} disabled={!dirty || saving}>
				{saving ? "Saving…" : "Save changes"}
			</button>
		</div>
	{/if}
</div>

<style>
	.settings {
		max-width: 720px;
		margin: 0 auto;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 18px;
	}
	.head h2 {
		margin: 0;
	}
	.row {
		display: flex;
		align-items: center;
	}
	.row.gap {
		gap: 8px;
		margin-top: 10px;
	}
	.kv-paths {
		display: grid;
		grid-template-columns: 140px 1fr;
		gap: 8px 16px;
	}
	.kv-paths .k {
		color: var(--fg-muted);
		font-size: 12.5px;
	}
	.kv-paths .v {
		font-size: 12.5px;
		color: var(--fg);
		word-break: break-all;
	}
	.block {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 12px;
		padding: 18px 20px;
		margin-bottom: 14px;
	}
	.block h3 {
		margin: 0 0 4px;
		font-size: 14px;
		font-weight: 700;
	}
	.block .muted {
		margin: 0 0 10px;
	}
	.list {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin: 8px 0;
	}
	.row-tag {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 999px;
		font-size: 12.5px;
	}
	.row-tag code {
		font-family: ui-monospace, Menlo, monospace;
		font-size: 12px;
	}
	.x {
		background: transparent;
		border: none;
		color: var(--fg-muted);
		cursor: pointer;
		font-size: 15px;
		line-height: 1;
		padding: 0 2px;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: 10px 16px;
	}
	.grid label {
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 12.5px;
		color: var(--fg-muted);
	}
	.grid label.check {
		flex-direction: row;
		align-items: center;
		gap: 6px;
		color: var(--fg);
	}
	.grid input,
	.grid select {
		padding: 6px 10px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--fg);
		font: inherit;
	}
	.grid input:focus,
	.grid select:focus {
		outline: 2px solid var(--accent-soft);
		border-color: var(--accent);
	}
	.actions {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 12px;
		position: sticky;
		bottom: 0;
		padding: 12px 0;
		background: var(--bg);
		border-top: 1px solid var(--border);
	}
	.unsaved {
		margin-right: auto;
		font-size: 12px;
		font-weight: 600;
		color: var(--warn);
	}
	.updater-row {
		align-items: center;
		justify-content: space-between;
	}
	.button-like {
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 7px 12px;
		color: var(--fg);
		text-decoration: none;
		font-size: 13px;
	}
	.danger-zone {
		border-color: color-mix(in srgb, var(--danger) 28%, var(--border));
	}
	.danger-zone h3 {
		color: var(--danger);
	}
	button.danger {
		background: var(--danger);
		color: #fff;
		border: none;
		padding: 8px 14px;
		border-radius: 8px;
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	button.danger:hover {
		background: color-mix(in srgb, var(--danger) 88%, #000);
	}
	.small {
		font-size: 12px;
	}
	.snap-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 4px;
	}
	.snap-head h3 {
		margin: 0;
	}
	.snap-list {
		list-style: none;
		margin: 12px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.snap-list li {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 12px;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 8px;
	}
	.snap-meta {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.snap-meta code {
		font-family: ui-monospace, Menlo, monospace;
		font-size: 12px;
	}
	.snap-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.snap-actions button {
		padding: 4px 10px;
		font-size: 12.5px;
	}
	input[type="text"] {
		flex: 1;
		padding: 6px 10px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--fg);
		font: inherit;
	}
	input[type="text"]:focus {
		outline: 2px solid var(--accent-soft);
		border-color: var(--accent);
	}
</style>
