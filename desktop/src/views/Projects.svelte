<script lang="ts">
	import { ask } from "@tauri-apps/plugin-dialog";
	import { onDestroy, onMount } from "svelte";
	import * as api from "../lib/api";
	import Select from "../components/Select.svelte";
	import {
		frameworkLabel,
		languageLabel,
		packageManagerLabel,
		relativeTime,
	} from "../lib/format";
	import { renderMarkdown } from "../lib/markdown";
	import {
		errorString,
		pushToast,
		selectedVobe,
		view,
		vobes,
	} from "../lib/stores";
	import type {
		ActivityEvent,
		EditorApp,
		TerminalApp,
		TodoHit,
	} from "../lib/types";

	let detailActivity: ActivityEvent[] = [];
	let notesDraft = "";
	let savingNotes = false;
	let busy = false;
	let readmeText: string | null = null;
	let readmeLoaded = false;
	let readmeLoading = false;
	let readmeMode: "preview" | "raw" = "preview";
	let readmeHtml = "";
	let todos: TodoHit[] = [];
	let todosLoaded = false;
	let todosLoading = false;
	let showTodos = false;
	let newTag = "";
	let copyingPack = false;
	let dirtyTimeout: number | undefined;
	let terminals: TerminalApp[] = [];
	let editors: EditorApp[] = [];
	let selectedTerminal: string | null = null;
	let selectedEditor: string | null = null;

	$: terminalOptions = terminals.map((t) => ({
		value: t.id,
		label: t.is_default ? `${t.label} (default)` : t.label,
	}));
	$: editorOptions = editors.map((e) => ({
		value: e.id,
		label: e.is_default ? `${e.label} (default)` : e.label,
	}));
	$: terminalLabel =
		terminals
			.find((t) => t.id === selectedTerminal)
			?.label.replace(/\s*\(default\)$/, "") ?? "Terminal";
	$: editorLabel =
		editors
			.find((e) => e.id === selectedEditor)
			?.label.replace(/\s*\(default\)$/, "") ?? "editor";

	$: selected = $selectedVobe;

	$: if (selected) {
		notesDraft = selected.notes ?? "";
		readmeText = null;
		readmeLoaded = false;
		readmeHtml = "";
		todos = [];
		todosLoaded = false;
		loadActivity(selected.id);
		if (dirtyTimeout) clearTimeout(dirtyTimeout);
	}

	onMount(async () => {
		try {
			[terminals, editors] = await Promise.all([
				api.listTerminals(),
				api.listEditors(),
			]);
			selectedTerminal =
				terminals.find((t) => t.is_default)?.id ??
				terminals[0]?.id ??
				null;
			selectedEditor =
				editors.find((e) => e.is_default)?.id ?? editors[0]?.id ?? null;
		} catch (e) {
			console.warn("list_terminals/editors failed:", e);
		}
	});

	async function loadActivity(id: string) {
		try {
			detailActivity = await api.vobeActivity(id, 50);
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		}
	}

	function back() {
		selectedVobe.set(null);
		view.set("dashboard");
	}

	async function doOpenInEditor(v: (typeof $vobes)[number]) {
		busy = true;
		try {
			await api.markOpened(v.name);
			await api.openInEditor(v.name, selectedEditor ?? undefined);
			pushToast({
				kind: "success",
				message: `Opened ${v.name} in editor.`,
			});
		} catch (e) {
			pushToast({ kind: "error", message: `Editor: ${errorString(e)}` });
		} finally {
			busy = false;
		}
	}

	async function doOpenTerminal(v: (typeof $vobes)[number]) {
		busy = true;
		try {
			await api.markOpened(v.name);
			await api.openInTerminalWith(v.name, selectedTerminal ?? "");
			pushToast({ kind: "success", message: `Terminal at ${v.name}.` });
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			busy = false;
		}
	}

	async function doReveal(v: (typeof $vobes)[number]) {
		busy = true;
		try {
			await api.revealInFinder(v.name);
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			busy = false;
		}
	}

	async function doCopyPath(v: (typeof $vobes)[number]) {
		try {
			await api.copyText(v.path);
			pushToast({ kind: "info", message: "Path copied." });
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		}
	}

	async function doCopyContextPack(v: (typeof $vobes)[number]) {
		copyingPack = true;
		try {
			const pack = await api.contextPack(v.name);
			await api.copyText(JSON.stringify(pack, null, 2));
			pushToast({
				kind: "success",
				message: `Context pack for ${v.name} copied (${pack.directory.length} entries, ${pack.activity.length} events).`,
			});
		} catch (e) {
			pushToast({
				kind: "error",
				message: `Context pack: ${errorString(e)}`,
			});
		} finally {
			copyingPack = false;
		}
	}

	async function togglePin(v: (typeof $vobes)[number]) {
		busy = true;
		try {
			await api.setPinned(v.name, !v.pinned);
			const fresh = $vobes.find((x) => x.id === v.id) ?? null;
			selectedVobe.set(fresh);
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			busy = false;
		}
	}

	async function removeVobe(v: (typeof $vobes)[number]) {
		const ok = await ask(
			`Remove ${v.name}? This only untracks it; the files are untouched.`,
			{ title: "Vobes", kind: "warning" },
		);
		if (!ok) {
			return;
		}
		busy = true;
		try {
			await api.removeVobe(v.name);
			pushToast({ kind: "success", message: `Removed ${v.name}.` });
			selectedVobe.set(null);
			view.set("dashboard");
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			busy = false;
		}
	}

	async function saveNotes() {
		if (!selected) return;
		savingNotes = true;
		try {
			const fresh = await api.saveNotes(
				selected.name,
				notesDraft.trim() || null,
			);
			selectedVobe.set(fresh);
			pushToast({ kind: "success", message: "Notes saved." });
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			savingNotes = false;
		}
	}

	async function addTag() {
		if (!selected) return;
		const t = newTag.trim();
		if (!t) return;
		if (selected.tags.includes(t)) {
			newTag = "";
			return;
		}
		busy = true;
		try {
			const fresh = await api.setTags(selected.name, [
				...selected.tags,
				t,
			]);
			selectedVobe.set(fresh);
			newTag = "";
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			busy = false;
		}
	}

	async function removeTag(t: string) {
		if (!selected) return;
		busy = true;
		try {
			const fresh = await api.setTags(
				selected.name,
				selected.tags.filter((x) => x !== t),
			);
			selectedVobe.set(fresh);
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			busy = false;
		}
	}

	async function loadReadme() {
		if (!selected || readmeLoaded || readmeLoading) return;
		readmeLoading = true;
		try {
			readmeText = await api.readReadme(selected.name);
			readmeHtml = readmeText ? renderMarkdown(readmeText) : "";
			readmeLoaded = true;
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			readmeLoading = false;
		}
	}

	function setReadmeMode(m: "preview" | "raw") {
		readmeMode = m;
	}

	async function openReadmeLink(e: MouseEvent) {
		const target = e.target as HTMLElement | null;
		if (!target) return;
		const anchor = target.closest("a");
		if (!anchor) return;
		e.preventDefault();
		const href = (anchor as HTMLAnchorElement).getAttribute("href");
		if (!href) return;
		try {
			if (
				/^https?:\/\//i.test(href) ||
				/^mailto:/i.test(href) ||
				/^tel:/i.test(href)
			) {
				await api.openUrlExternal(href);
			} else {
				await api.openPathExternal(href);
			}
		} catch (err) {
			pushToast({ kind: "error", message: errorString(err) });
		}
	}

	async function loadTodos() {
		if (!selected || todosLoaded || todosLoading) return;
		todosLoading = true;
		try {
			todos = await api.scrapeTodos(selected.name);
			todosLoaded = true;
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		} finally {
			todosLoading = false;
		}
	}

	function toggleTodos() {
		showTodos = !showTodos;
		if (showTodos) loadTodos();
	}

	onDestroy(() => {
		if (dirtyTimeout) clearTimeout(dirtyTimeout);
	});
</script>

<div class="projects">
	{#if !selected}
		<div class="empty-state">
			<div class="empty-card">
				<div class="empty-icon" aria-hidden="true">↗</div>
				<h2>Pick a project to inspect</h2>
				<p>
					The Dashboard is the fastest way to find something — search,
					sort, or scan the attention section.
				</p>
				<div class="empty-actions">
					<button
						class="primary"
						on:click={() => view.set("dashboard")}
					>
						Go to Dashboard
					</button>
				</div>
				<div class="empty-hint">
					Or press <kbd>2</kbd> to jump straight here, <kbd>⌘K</kbd> to
					fuzzy-find a vobe.
				</div>
			</div>
		</div>
	{:else}
		<div class="head">
			<button class="back" on:click={back}>← Dashboard</button>
			<h2>{selected.name}</h2>
			<div class="row">
				<button on:click={() => togglePin(selected)} disabled={busy}>
					{selected.pinned ? "Unpin" : "Pin"}
				</button>
				<button on:click={() => doCopyPath(selected)} disabled={busy}
					>Copy path</button
				>
				<button on:click={() => doReveal(selected)} disabled={busy}
					>Reveal</button
				>
				<div class="with-select">
					<button
						on:click={() => doOpenTerminal(selected)}
						disabled={busy}>{terminalLabel}</button
					>
					{#if terminalOptions.length > 0}
						<Select
							value={selectedTerminal ?? ""}
							options={terminalOptions}
							ariaLabel="Choose terminal"
							compact={true}
							menuAlign="right"
							onChange={(v) => (selectedTerminal = v)}
						/>
					{/if}
				</div>
				<button
					on:click={() => doCopyContextPack(selected)}
					disabled={busy || copyingPack}
				>
					{copyingPack ? "Copying…" : "Copy context pack"}
				</button>
				<div class="with-select">
					<button
						class="primary"
						on:click={() => doOpenInEditor(selected)}
						disabled={busy}
					>
						Open in {editorLabel}
					</button>
					{#if editorOptions.length > 0}
						<Select
							value={selectedEditor ?? ""}
							options={editorOptions}
							ariaLabel="Choose editor"
							compact={true}
							menuAlign="right"
							onChange={(v) => (selectedEditor = v)}
						/>
					{/if}
				</div>
			</div>
		</div>

		<div class="detail">
			<section class="card">
				<div class="kv">
					<div class="k">Path</div>
					<div class="v mono">{selected.path}</div>
					<div class="k">Language</div>
					<div class="v" class:vanilla={!selected.language}>
						{languageLabel(selected.language)}
					</div>
					<div class="k">Framework</div>
					<div class="v" class:vanilla={!selected.framework}>
						{frameworkLabel(selected.framework)}
					</div>
					<div class="k">Package manager</div>
					<div class="v" class:vanilla={!selected.package_manager}>
						{packageManagerLabel(selected.package_manager)}
					</div>
					<div class="k">Tags</div>
					<div class="v tags-cell">
						{#each selected.tags as t (t)}
							<span class="tag">
								{t}
								<button
									type="button"
									class="tag-x"
									aria-label="Remove tag {t}"
									on:click={() => removeTag(t)}
									disabled={busy}>×</button
								>
							</span>
						{/each}
						<input
							class="tag-input"
							type="text"
							bind:value={newTag}
							placeholder="add tag…"
							on:keydown={(e) => {
								if (e.key === "Enter") addTag();
							}}
						/>
					</div>
					<div class="k">Last opened</div>
					<div class="v">{relativeTime(selected.last_opened)}</div>
					<div class="k">Last modified</div>
					<div class="v">{relativeTime(selected.last_modified)}</div>
				</div>
			</section>

			{#if selected.git}
				<section class="card">
					<h3>Git</h3>
					<div class="kv">
						<div class="k">Branch</div>
						<div class="v mono">{selected.git.branch}</div>
						<div class="k">Status</div>
						<div class="v">
							{selected.git.dirty ? "dirty" : "clean"}
							{#if selected.git.ahead > 0}
								· ↑{selected.git.ahead} to push{/if}
							{#if selected.git.behind > 0}
								· ↓{selected.git.behind} behind{/if}
						</div>
						{#if selected.git.last_commit}
							<div class="k">Last commit</div>
							<div class="v">
								<div class="mono small">
									{selected.git.last_commit.hash.slice(0, 7)}
								</div>
								<div>{selected.git.last_commit.message}</div>
								<div class="muted small">
									{selected.git.last_commit.author} · {relativeTime(
										selected.git.last_commit.date,
									)}
								</div>
							</div>
						{/if}
					</div>
				</section>
			{/if}

			<section class="card span2">
				<div class="row-between">
					<h3>TODO / FIXME</h3>
					{#if !todosLoaded && !todosLoading}
						<button class="link" on:click={loadTodos}>Scan</button>
					{:else if todosLoading}
						<span class="muted small">scanning…</span>
					{:else}
						<button class="link" on:click={toggleTodos}>
							{showTodos ? "Hide" : `Show ${todos.length}`}
						</button>
					{/if}
				</div>
				{#if todosLoaded && !showTodos}
					<div class="muted small">{todos.length} found.</div>
				{/if}
				{#if showTodos && todos.length > 0}
					<ul class="todos">
						{#each todos.slice(0, 20) as t (t.file + ":" + t.line + t.text)}
							<li>
								<span class="kind kind-{t.kind.toLowerCase()}"
									>{t.kind}</span
								>
								<span class="loc mono">{t.file}:{t.line}</span>
								<span class="text">{t.text}</span>
							</li>
						{/each}
						{#if todos.length > 20}
							<li class="muted small">
								… and {todos.length - 20} more
							</li>
						{/if}
					</ul>
				{/if}
			</section>

			<section class="card span2">
				<div class="row-between">
					<h3>README</h3>
					{#if readmeLoaded && readmeText}
						<div
							class="readme-toggle"
							role="tablist"
							aria-label="README view"
						>
							<button
								type="button"
								role="tab"
								aria-selected={readmeMode === "preview"}
								class:active={readmeMode === "preview"}
								on:click={() => setReadmeMode("preview")}
								>Preview</button
							>
							<button
								type="button"
								role="tab"
								aria-selected={readmeMode === "raw"}
								class:active={readmeMode === "raw"}
								on:click={() => setReadmeMode("raw")}
								>Raw</button
							>
						</div>
					{:else if !readmeLoaded && !readmeLoading}
						<button class="link" on:click={loadReadme}>Load</button>
					{:else if readmeLoading}
						<span class="muted small">loading…</span>
					{/if}
				</div>
				{#if readmeLoaded}
					{#if !readmeText}
						<div class="muted small">
							No README found in this project.
						</div>
					{:else if readmeMode === "preview"}
						<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
						<!-- svelte-ignore a11y-click-events-have-key-events -->
						<!-- eslint-disable-next-line svelte/no-at-html-tags -->
						<div
							class="md"
							on:click={openReadmeLink}
							role="document"
						>
							{@html readmeHtml}
						</div>
					{:else}
						<pre class="readme">{readmeText}</pre>
					{/if}
				{/if}
			</section>

			<section class="card span2">
				<h3>Notes</h3>
				<textarea
					bind:value={notesDraft}
					rows="5"
					placeholder="Markdown allowed. Pick up where you left off."
				></textarea>
				<div class="actions">
					<button on:click={() => (notesDraft = selected.notes ?? "")}
						>Reset</button
					>
					<button
						class="primary"
						on:click={saveNotes}
						disabled={savingNotes}
					>
						{savingNotes ? "Saving…" : "Save notes"}
					</button>
				</div>
			</section>

			<section class="card span2">
				<h3>Activity</h3>
				{#if detailActivity.length === 0}
					<div class="muted">No recorded activity yet.</div>
				{:else}
					<ul class="timeline">
						{#each detailActivity as e (e.id ?? `${e.vobe_id}-${e.timestamp}`)}
							<li>
								<span class="when"
									>{relativeTime(e.timestamp)}</span
								>
								<span class="what">{e.kind.toLowerCase()}</span>
								{#if e.detail}<span class="detail"
										>— {e.detail}</span
									>{/if}
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<section class="card span2 danger-zone">
				<h3>Danger zone</h3>
				<button
					class="danger"
					on:click={() => removeVobe(selected)}
					disabled={busy}
				>
					Untrack this project
				</button>
				<div class="muted small">
					Vobes forgets this project but does not touch the files on
					disk.
				</div>
			</section>
		</div>
	{/if}
</div>

<style>
	.projects {
		max-width: 1000px;
		margin: 0 auto;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 16px;
		flex-wrap: wrap;
	}
	.head h2 {
		margin: 0;
		flex: 1;
		min-width: 160px;
	}
	.row {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.back {
		background: transparent;
		border: none;
		color: var(--fg-muted);
		cursor: pointer;
		padding: 4px 6px;
		font: inherit;
	}
	.back:hover {
		color: var(--fg);
	}
	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 60vh;
	}
	.empty-card {
		max-width: 440px;
		text-align: center;
		background: var(--bg-elevated);
		border: 1px dashed var(--border);
		border-radius: 16px;
		padding: 36px 32px;
	}
	.empty-icon {
		width: 44px;
		height: 44px;
		margin: 0 auto 14px;
		display: grid;
		place-items: center;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 12px;
		color: var(--accent);
		font-size: 20px;
	}
	.empty-card h2 {
		margin: 0 0 6px;
		font-size: 17px;
		letter-spacing: -0.01em;
	}
	.empty-card p {
		margin: 0 0 18px;
		color: var(--fg-muted);
		font-size: 13.5px;
		line-height: 1.55;
	}
	.empty-actions {
		display: flex;
		justify-content: center;
		gap: 8px;
		margin-bottom: 14px;
	}
	.empty-hint {
		font-size: 12px;
		color: var(--fg-faint);
	}
	.empty-hint kbd {
		font-family: ui-monospace, Menlo, monospace;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 0 5px;
		font-size: 11px;
	}
	.detail {
		display: grid;
		gap: 14px;
		grid-template-columns: 1fr 1fr;
	}
	.detail > .span2 {
		grid-column: 1 / -1;
	}
	@media (max-width: 800px) {
		.detail {
			grid-template-columns: 1fr;
		}
	}
	.card {
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 12px;
		padding: 16px 18px;
	}
	.card h3 {
		margin: 0 0 10px;
		font-size: 13px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--fg-faint);
	}
	.row-between {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}
	.row-between h3 {
		margin: 0;
	}
	.link {
		background: transparent;
		border: none;
		cursor: pointer;
		color: var(--accent);
		padding: 0;
		font: inherit;
		text-decoration: underline;
		font-size: 12px;
	}
	.kv {
		display: grid;
		grid-template-columns: 130px 1fr;
		gap: 8px 16px;
	}
	.k {
		color: var(--fg-muted);
		font-size: 12.5px;
	}
	.v {
		font-size: 13.5px;
	}
	.v.mono {
		font-family: ui-monospace, Menlo, monospace;
		font-size: 12.5px;
	}
	.small {
		font-size: 12px;
	}
	.muted {
		color: var(--fg-muted);
	}
	.vanilla {
		color: var(--fg-faint);
	}
	.tag {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 4px 2px 8px;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 999px;
		font-size: 11.5px;
		color: var(--fg);
		margin-right: 4px;
		margin-bottom: 4px;
	}
	.tag-x {
		background: transparent;
		border: none;
		color: var(--fg-muted);
		cursor: pointer;
		font-size: 14px;
		line-height: 1;
		padding: 0 4px;
		border-radius: 50%;
	}
	.tag-x:hover {
		color: var(--danger);
	}
	.tag-input {
		background: transparent;
		border: 1px dashed var(--border);
		border-radius: 999px;
		padding: 2px 10px;
		color: var(--fg);
		font: inherit;
		font-size: 11.5px;
		width: 100px;
	}
	.tag-input:focus {
		outline: none;
		border-color: var(--accent);
	}
	.tags-cell {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px;
	}
	textarea {
		width: 100%;
		padding: 8px 10px;
		border: 1px solid var(--border);
		border-radius: 8px;
		background: var(--bg);
		color: var(--fg);
		font: inherit;
		resize: vertical;
	}
	textarea:focus {
		outline: 2px solid var(--accent-soft);
		border-color: var(--accent);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}
	.with-select {
		display: inline-flex;
		align-items: stretch;
		gap: 0;
	}
	.with-select > button:first-child {
		border-top-right-radius: 0;
		border-bottom-right-radius: 0;
		border-right: none;
	}
	.with-select :global(.select.compact) {
		display: inline-flex;
	}
	.with-select :global(.select.compact .trigger) {
		border-top-left-radius: 0;
		border-bottom-left-radius: 0;
		padding: 0 8px;
		min-width: 26px;
		background: var(--bg-elevated);
	}
	.with-select > button.primary + :global(.select.compact .trigger) {
		background: var(--accent-soft);
		color: var(--accent);
	}
	.timeline {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.timeline li {
		display: flex;
		gap: 10px;
		align-items: center;
		padding: 6px 0;
		border-bottom: 1px solid var(--border);
		font-size: 13px;
	}
	.timeline li:last-child {
		border-bottom: none;
	}
	.timeline .when {
		color: var(--fg-faint);
		min-width: 80px;
	}
	.timeline .what {
		font-weight: 600;
	}
	.timeline .detail {
		color: var(--fg-muted);
	}
	.readme {
		margin: 0;
		padding: 12px 14px;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 8px;
		font-family: ui-monospace, Menlo, monospace;
		font-size: 12px;
		line-height: 1.5;
		color: var(--fg);
		max-height: 280px;
		overflow-y: auto;
		white-space: pre-wrap;
		word-break: break-word;
	}
	.readme-toggle {
		display: inline-flex;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 6px;
		padding: 2px;
		gap: 2px;
	}
	.readme-toggle button {
		background: transparent;
		border: none;
		color: var(--fg-muted);
		font: inherit;
		font-size: 11.5px;
		font-weight: 500;
		padding: 2px 10px;
		border-radius: 4px;
		cursor: pointer;
	}
	.readme-toggle button.active {
		background: var(--bg-elevated);
		color: var(--fg);
		box-shadow: var(--shadow-sm);
	}
	.readme-toggle button:hover:not(.active) {
		color: var(--fg);
	}
	.md {
		padding: 14px 18px;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 8px;
		color: var(--fg);
		font-size: 13.5px;
		line-height: 1.6;
		max-height: 360px;
		overflow-y: auto;
	}
	.md :global(h1),
	.md :global(h2),
	.md :global(h3),
	.md :global(h4) {
		margin: 18px 0 8px;
		line-height: 1.3;
		letter-spacing: -0.01em;
	}
	.md :global(h1) {
		font-size: 20px;
		font-weight: 700;
		padding-bottom: 6px;
		border-bottom: 1px solid var(--border);
	}
	.md :global(h2) {
		font-size: 16px;
		font-weight: 700;
		padding-bottom: 4px;
		border-bottom: 1px solid var(--border);
	}
	.md :global(h3) {
		font-size: 14px;
		font-weight: 700;
	}
	.md :global(h4) {
		font-size: 13px;
		font-weight: 600;
		color: var(--fg-muted);
	}
	.md :global(p) {
		margin: 8px 0;
	}
	.md :global(ul),
	.md :global(ol) {
		margin: 8px 0;
		padding-left: 22px;
	}
	.md :global(li) {
		margin: 3px 0;
	}
	.md :global(a) {
		color: var(--accent);
		text-decoration: none;
	}
	.md :global(a:hover) {
		text-decoration: underline;
	}
	.md :global(code) {
		font-family: ui-monospace, Menlo, monospace;
		font-size: 12px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 1px 5px;
	}
	.md :global(pre) {
		margin: 10px 0;
		padding: 10px 12px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 6px;
		overflow-x: auto;
	}
	.md :global(pre code) {
		background: transparent;
		border: none;
		padding: 0;
		font-size: 12px;
	}
	.md :global(blockquote) {
		margin: 10px 0;
		padding: 6px 12px;
		border-left: 3px solid var(--accent);
		color: var(--fg-muted);
		background: var(--bg);
		border-radius: 0 4px 4px 0;
	}
	.md :global(hr) {
		border: none;
		border-top: 1px solid var(--border);
		margin: 16px 0;
	}
	.md :global(table) {
		border-collapse: collapse;
		margin: 10px 0;
		font-size: 12.5px;
		width: 100%;
	}
	.md :global(th),
	.md :global(td) {
		border: 1px solid var(--border);
		padding: 6px 10px;
		text-align: left;
	}
	.md :global(th) {
		background: var(--bg);
		font-weight: 600;
	}
	.md :global(img) {
		max-width: 100%;
		height: auto;
		border-radius: 6px;
	}
	.todos {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.todos li {
		display: flex;
		align-items: baseline;
		gap: 10px;
		padding: 5px 0;
		border-bottom: 1px solid var(--border);
		font-size: 12.5px;
	}
	.todos li:last-child {
		border-bottom: none;
	}
	.todos .kind {
		font-weight: 700;
		font-size: 10.5px;
		padding: 1px 6px;
		border-radius: 4px;
		flex: none;
		min-width: 50px;
		text-align: center;
	}
	.kind-todo {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		color: var(--accent);
	}
	.kind-fixme {
		background: color-mix(in srgb, var(--danger) 18%, transparent);
		color: var(--danger);
	}
	.kind-xxx {
		background: color-mix(in srgb, var(--warn) 18%, transparent);
		color: var(--warn);
	}
	.todos .loc {
		color: var(--fg-faint);
		font-size: 11.5px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 260px;
	}
	.todos .text {
		color: var(--fg);
		flex: 1;
		min-width: 0;
	}
	.danger-zone h3 {
		color: var(--danger);
	}
	.danger {
		background: transparent;
		color: var(--danger);
		border-color: var(--danger);
	}
	.mono {
		font-family: ui-monospace, Menlo, monospace;
	}
</style>
