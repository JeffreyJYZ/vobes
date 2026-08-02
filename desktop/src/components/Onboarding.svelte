<script lang="ts">
	import { open as openDialog } from "@tauri-apps/plugin-dialog";
	import { onMount } from "svelte";
	import {
		applyConfig,
		config,
		configPath,
		doScan,
		errorString,
		loadConfig,
		onboardingDone,
		pushToast,
		refresh,
		view,
		vobes,
	} from "../lib/stores";
	import type { ScanConfig } from "../lib/types";

	let step: 1 | 2 | 3 = 1;
	let roots: string[] = [];
	let excludes = "";
	let maxDepth = 4;
	let followSymlinks = false;
	let scanning = false;
	let scanError = "";

	onMount(async () => {
		await loadConfig();
		if ($config) {
			roots = [...$config.scan.roots];
			excludes = $config.scan.exclude.join(", ");
			maxDepth = $config.scan.max_depth;
			followSymlinks = $config.scan.follow_symlinks;
		}
	});

	async function pickRoot() {
		try {
			const picked = await openDialog({
				directory: true,
				multiple: false,
			});
			if (picked) {
				const p = String(picked);
				if (!roots.includes(p)) roots = [...roots, p];
			}
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		}
	}

	function removeRoot(r: string) {
		roots = roots.filter((x) => x !== r);
	}

	async function save() {
		if (!$config) return;
		const next: ScanConfig = {
			roots,
			exclude: excludes
				.split(",")
				.map((s) => s.trim())
				.filter(Boolean),
			max_depth: Math.max(1, maxDepth | 0),
			follow_symlinks: followSymlinks,
		};
		const updated = { ...$config, scan: next };
		try {
			await applyConfig(updated);
			step = 3;
			scanning = true;
			scanError = "";
			try {
				await refresh();
				await doScan();
			} catch (e) {
				scanError = errorString(e);
			} finally {
				scanning = false;
			}
		} catch (e) {
			pushToast({ kind: "error", message: errorString(e) });
		}
	}

	function finish() {
		if (typeof localStorage !== "undefined") {
			localStorage.setItem("vobes:onboarded", "1");
		}
		onboardingDone.set(true);
		view.set("dashboard");
	}

	function onRootPathKeydown(e: KeyboardEvent) {
		if (e.key !== "Enter") return;
		const t = e.currentTarget as HTMLInputElement;
		if (t.value && !roots.includes(t.value)) {
			roots = [...roots, t.value];
		}
		t.value = "";
	}
</script>

<div class="onboard">
	<div class="card">
		<div class="head">
			<div class="step-row">
				<span class="dot" class:on={step >= 1}></span>
				<span class="dot" class:on={step >= 2}></span>
				<span class="dot" class:on={step >= 3}></span>
			</div>
			<h2>Welcome to Vobes</h2>
			<p class="lede">
				A calm home for every project you touch. Let's set it up in 30
				seconds.
			</p>
		</div>

		{#if step === 1}
			<div class="step">
				<h3>1. Where do your projects live?</h3>
				<p class="muted">
					Pick the folders Vobes should scan. You can change these any
					time in Settings. Use <code>~/dev</code> for the home-relative
					shorthand, or pick actual folders.
				</p>
				<div class="roots">
					{#each roots as r (r)}
						<div class="root-row">
							<code>{r}</code>
							<button
								class="x"
								on:click={() => removeRoot(r)}
								aria-label="Remove">×</button
							>
						</div>
					{/each}
					{#if roots.length === 0}
						<div class="muted small">
							No folders yet. Add one below.
						</div>
					{/if}
				</div>
				<div class="row gap">
					<button class="primary" on:click={pickRoot}
						>Add folder…</button
					>
					<input
						class="text"
						type="text"
						placeholder="or paste a path, e.g. ~/work"
						on:keydown={onRootPathKeydown}
					/>
				</div>
				<div class="row gap">
					<label class="num"
						>Max depth
						<input
							type="number"
							min="1"
							max="10"
							bind:value={maxDepth}
						/>
					</label>
					<label class="num"
						>Excludes (comma)
						<input
							type="text"
							bind:value={excludes}
							placeholder="scratch, experiments"
						/>
					</label>
					<label class="check">
						<input type="checkbox" bind:checked={followSymlinks} />
						Follow symlinks
					</label>
				</div>
				<div class="actions">
					<button on:click={() => view.set("dashboard")}
						>Skip for now</button
					>
					<button
						class="primary"
						on:click={save}
						disabled={roots.length === 0}
					>
						Next →
					</button>
				</div>
			</div>
		{:else if step === 2}
			<div class="step">
				<h3>2. Saving…</h3>
				<p class="muted">
					Writing config to <code
						>{$configPath || "your config file"}</code
					>.
				</p>
			</div>
		{:else if step === 3}
			<div class="step">
				<h3>3. Scanning your projects</h3>
				{#if scanning}
					<p class="muted">
						Walking {$config?.scan.roots.length ?? 0} roots… this can
						take a moment on a big workspace.
					</p>
					<div class="bar"><div class="bar-fill"></div></div>
				{:else if scanError}
					<p class="err">Scan hit a snag: {scanError}</p>
					<div class="actions">
						<button on:click={() => (step = 1)}>← Back</button>
						<button class="primary" on:click={finish}
							>Continue</button
						>
					</div>
				{:else}
					<p>
						Discovered <strong>{$vobes.length}</strong>
						vobe{$vobes.length === 1 ? "" : "s"}.
					</p>
					<div class="actions">
						<button class="primary" on:click={finish}
							>Open dashboard →</button
						>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.onboard {
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 24px;
	}
	.card {
		width: min(620px, 100%);
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 16px;
		padding: 32px 32px 28px;
		box-shadow: var(--shadow-md);
	}
	.head h2 {
		margin: 12px 0 4px;
		font-size: 22px;
		letter-spacing: -0.02em;
	}
	.lede {
		margin: 0 0 24px;
		color: var(--fg-muted);
		font-size: 14px;
	}
	.step h3 {
		margin: 0 0 10px;
		font-size: 16px;
	}
	.muted {
		color: var(--fg-muted);
		font-size: 13px;
		line-height: 1.55;
	}
	.small {
		font-size: 12.5px;
	}
	.err {
		color: var(--danger);
		font-size: 13px;
	}
	.step-row {
		display: flex;
		gap: 6px;
	}
	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
	}
	.dot.on {
		background: var(--accent);
		border-color: var(--accent);
	}
	.roots {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin: 14px 0;
	}
	.root-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 8px;
		font-family: ui-monospace, Menlo, monospace;
		font-size: 12.5px;
	}
	.x {
		background: transparent;
		border: none;
		cursor: pointer;
		color: var(--fg-muted);
		font-size: 16px;
		padding: 0 4px;
	}
	.row {
		display: flex;
		align-items: center;
	}
	.gap {
		gap: 10px;
		margin-top: 10px;
	}
	input.text {
		flex: 1;
		padding: 7px 10px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		color: var(--fg);
		font: inherit;
	}
	input.text:focus {
		outline: 2px solid var(--accent-soft);
		border-color: var(--accent);
	}
	label.num,
	label.check {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 12.5px;
		color: var(--fg-muted);
	}
	label.num input[type="number"],
	label.num input[type="text"] {
		width: 130px;
		padding: 6px 8px;
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 6px;
		color: var(--fg);
		font: inherit;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 22px;
	}
	.bar {
		height: 6px;
		background: var(--bg-sunken);
		border-radius: 999px;
		overflow: hidden;
		margin: 16px 0;
	}
	.bar-fill {
		height: 100%;
		background: var(--accent);
		width: 40%;
		animation: slide 1.1s ease-in-out infinite alternate;
	}
	@keyframes slide {
		from {
			margin-left: 0%;
		}
		to {
			margin-left: 60%;
		}
	}
	code {
		font-family: ui-monospace, Menlo, monospace;
		background: var(--bg-sunken);
		padding: 1px 5px;
		border-radius: 4px;
		font-size: 12px;
	}
</style>
