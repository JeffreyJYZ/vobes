<script lang="ts">
	import { listen } from "@tauri-apps/api/event";
	import { onDestroy, onMount } from "svelte";
	import CommandPalette from "./components/CommandPalette.svelte";
	import Onboarding from "./components/Onboarding.svelte";
	import Settings from "./components/Settings.svelte";
	import ShortcutHelp from "./components/ShortcutHelp.svelte";
	import Sidebar from "./components/Sidebar.svelte";
	import Toast from "./components/Toast.svelte";
	import { modKeyLabel } from "./lib/format";
	import { isTypingTarget, matchShortcut, shortcuts } from "./lib/keyboard";
	import {
		addSavedFilter,
		applyTheme,
		config,
		doScan,
		effectiveTheme,
		helpOpen,
		loadConfig,
		loadSavedFilters,
		onboardingDone,
		openPalette,
		pushToast,
		refresh,
		selectedVobe,
		themePref,
		view,
		vobes,
	} from "./lib/stores";
	import { addVobe } from "./lib/api";
	import { checkForUpdate } from "./lib/updates";
	import Activity from "./views/Activity.svelte";
	import Dashboard from "./views/Dashboard.svelte";
	import Projects from "./views/Projects.svelte";

	let booted = false;
	let autoRefreshTimer: number | undefined;
	let fsDebounce: number | undefined;
	const unlistens: Array<() => void> = [];

	// The floating "⌘K Search" hint. Shown until the user either opens
	// the palette once, or dismisses it explicitly. Stops competing with
	// the sidebar's "Finish setup" button for the same corner.
	const PALETTE_HINT_KEY = "vobes:palette-hint-dismissed";
	let showPaletteHint = false;
	if (typeof localStorage !== "undefined") {
		showPaletteHint = localStorage.getItem(PALETTE_HINT_KEY) !== "1";
	}
	function dismissPaletteHint() {
		showPaletteHint = false;
		if (typeof localStorage !== "undefined") {
			localStorage.setItem(PALETTE_HINT_KEY, "1");
		}
	}

	// React to theme changes any time — initial load, settings save,
	// or live config swap. Keeps light/dark in sync without restart.
	$: syncThemeFromConfig($config);

	function syncThemeFromConfig(c: typeof $config) {
		if (!c) return;
		themePref.set(c.display.theme);
		const pref = c.display.theme;
		if (pref === "light" || pref === "dark") {
			effectiveTheme.set(pref);
		} else {
			// auto: read from OS
			if (typeof window !== "undefined" && window.matchMedia) {
				const mq = window.matchMedia("(prefers-color-scheme: dark)");
				effectiveTheme.set(mq.matches ? "dark" : "light");
			}
		}
	}

	onMount(async () => {
		// Apply theme immediately so the boot screen matches the user's pref.
		applyTheme();

		await loadConfig();
		syncThemeFromConfig($config);
		await refresh();
		await loadSavedFilters();
		booted = true;

		// Check for app updates in the background. Surface as a
		// toast with an action button — the user always opts in.
		void checkForUpdate();

		// Auto-refresh on window focus. We deliberately throttle to once per
		// 15s so a fast tab-switch doesn't hammer the disk.
		let lastFocusRefresh = 0;
		window.addEventListener("focus", () => {
			const now = Date.now();
			if (now - lastFocusRefresh > 15000) {
				lastFocusRefresh = now;
				refresh({ silent: true });
			}
		});

		// Background interval: if the user leaves Vobes open, keep git state
		// fresh. Disabled in the help / palette overlays so the UI stays calm.
		autoRefreshTimer = window.setInterval(() => {
			if (document.hasFocus()) refresh({ silent: true });
		}, 90000);

		// File-system watcher events from the Rust side. Coalesce a burst of
		// events (an editor save can produce dozens) into one refresh.
		unlistens.push(
			await listen<number>("vobes://fs-changed", () => {
				if (fsDebounce) clearTimeout(fsDebounce);
				fsDebounce = window.setTimeout(() => {
					fsDebounce = undefined;
					refresh({ silent: true });
				}, 600);
			}),
		);

		// Global shortcut: open the palette.
		unlistens.push(
			await listen("vobes://show-palette", () => {
				openPalette();
			}),
		);

		// Deep links: vobes://open/<name>
		unlistens.push(
			await listen<string[]>("deep-link://new-url", (e) => {
				for (const url of e.payload) handleDeepLink(url);
			}),
		);
	});

	onDestroy(() => {
		if (autoRefreshTimer) clearInterval(autoRefreshTimer);
		if (fsDebounce) clearTimeout(fsDebounce);
		for (const u of unlistens) u();
	});

	function handleDeepLink(url: string) {
		// Expected shapes:
		//   vobes://open/<name>     — focus vobe by name
		//   vobes://open?id=<id>    — focus vobe by stable id (preferred)
		//   vobes://add?path=<p>    — add vobe at path, then focus
		//   vobes://search?q=<q>    — save + focus a search query
		try {
			const u = new URL(url);
			if (u.hostname === "open" || u.pathname.startsWith("/open")) {
				const id = u.searchParams.get("id");
				const segs = u.pathname.split("/").filter(Boolean);
				const name = segs[0] ? decodeURIComponent(segs[0]) : "";
				const v = id
					? $vobes.find((v) => v.id === id)
					: $vobes.find(
							(v) =>
								v.name === name ||
								v.name.toLowerCase() === name.toLowerCase(),
						);
				if (v) {
					selectedVobe.set(v);
					view.set("projects");
				} else if (id) {
					pushToast({
						kind: "error",
						message: `No vobe with id "${id}".`,
					});
				} else {
					pushToast({
						kind: "error",
						message: `No vobe named "${name}".`,
					});
				}
			} else if (u.hostname === "add") {
				const path = u.searchParams.get("path");
				if (!path) {
					pushToast({
						kind: "error",
						message: "vobes://add needs ?path=<path>",
					});
					return;
				}
				addVobe(path)
					.then((v) => {
						selectedVobe.set(v);
						view.set("projects");
						refresh();
					})
					.catch((e: unknown) => {
						const msg =
							e && typeof e === "object" && "message" in e
								? String((e as { message: unknown }).message)
								: String(e);
						pushToast({
							kind: "error",
							message: `Could not add "${path}": ${msg}`,
						});
					});
			} else if (u.hostname === "search") {
				const q = u.searchParams.get("q") ?? "";
				view.set("dashboard");
				addSavedFilter(`Search: ${q}`, q).catch(() => {
					pushToast({ kind: "error", message: `Could not save search “${q}”.` });
				});
			} else {
				pushToast({ kind: "info", message: `Opened ${url}` });
			}
		} catch (_err) {
			pushToast({ kind: "error", message: `Bad deep link: ${url}` });
		}
	}

	function onGlobalKey(e: KeyboardEvent) {
		if (isTypingTarget(e)) return;

		// Palette is handled inside CommandPalette so it can scope its own keys.
		// The shortcut help toggle, view switching, and refresh live here.
		if (matchShortcut(shortcuts.find((s) => s.id === "help")!.combo, e)) {
			e.preventDefault();
			helpOpen.update((v) => !v);
			return;
		}
		if (
			matchShortcut(shortcuts.find((s) => s.id === "dashboard")!.combo, e)
		) {
			e.preventDefault();
			view.set("dashboard");
			return;
		}
		if (
			matchShortcut(shortcuts.find((s) => s.id === "projects")!.combo, e)
		) {
			e.preventDefault();
			// Projects view is detail-only — fall back to dashboard if no
			// vobe is currently selected.
			if (!$selectedVobe) {
				view.set("dashboard");
			} else {
				view.set("projects");
			}
			return;
		}
		if (
			matchShortcut(shortcuts.find((s) => s.id === "activity")!.combo, e)
		) {
			e.preventDefault();
			view.set("activity");
			return;
		}
		if (
			matchShortcut(shortcuts.find((s) => s.id === "settings")!.combo, e)
		) {
			e.preventDefault();
			view.set("settings");
			return;
		}
		if (matchShortcut(shortcuts.find((s) => s.id === "scan")!.combo, e)) {
			e.preventDefault();
			doScan();
			return;
		}
	}

	$: needsOnboarding = booted && !$onboardingDone && $vobes.length === 0;
</script>

<svelte:window on:keydown={onGlobalKey} />

<div class="app">
	<Sidebar />
	<main class="main">
		{#if !booted}
			<div class="boot">
				<div class="spinner"></div>
				<span>Loading vobes…</span>
			</div>
		{:else if needsOnboarding}
			<Onboarding />
		{:else if $view === "dashboard"}
			<Dashboard />
		{:else if $view === "projects"}
			<Projects />
		{:else if $view === "activity"}
			<Activity />
		{:else if $view === "settings"}
			<Settings />
		{/if}
	</main>
</div>

<CommandPalette />
<ShortcutHelp />
<Toast />

{#if showPaletteHint}
	<button
		class="palette-hint"
		type="button"
		on:click={() => {
			openPalette();
			dismissPaletteHint();
		}}
		title="Command palette"
	>
		<span class="kbd">{modKeyLabel()} K</span>
		<span class="lbl">Search</span>
		<button
			class="dismiss"
			type="button"
			aria-label="Dismiss hint"
			on:click|stopPropagation={dismissPaletteHint}>×</button
		>
	</button>
{/if}

<style>
	.app {
		display: grid;
		grid-template-columns: 232px 1fr;
		height: 100vh;
		background: var(--bg);
	}
	.main {
		padding: 32px 36px;
		overflow-y: auto;
		min-width: 0;
	}
	.boot {
		display: flex;
		align-items: center;
		gap: 12px;
		color: var(--fg-muted);
		padding: 60px 0;
	}
	.spinner {
		width: 16px;
		height: 16px;
		border: 2px solid var(--border);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.palette-hint {
		position: fixed;
		bottom: 18px;
		left: 250px;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 6px 5px 10px;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: 999px;
		color: var(--fg-muted);
		font-size: 12px;
		cursor: pointer;
		box-shadow: var(--shadow-sm);
		z-index: 40;
	}
	.palette-hint:hover {
		color: var(--fg);
	}
	.palette-hint .kbd {
		font-family: ui-monospace, Menlo, monospace;
		font-size: 11px;
		background: var(--bg-sunken);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 0 5px;
	}
	.palette-hint .dismiss {
		background: transparent;
		border: none;
		color: var(--fg-faint);
		cursor: pointer;
		font-size: 14px;
		line-height: 1;
		padding: 0 6px;
		border-radius: 50%;
	}
	.palette-hint .dismiss:hover {
		color: var(--fg);
	}
</style>
