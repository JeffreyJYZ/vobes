<script lang="ts">
import { invoke } from "@tauri-apps/api/core"
import { open as openDialog } from "@tauri-apps/plugin-dialog"
import { onMount, tick } from "svelte"
import { frameworkLabel, languageLabel, modKeyLabel } from "../lib/format"
import { fuzzy } from "../lib/fuzzy"
import { matchShortcut } from "../lib/keyboard"
import {
	addSavedFilter,
	closePalette,
	doScan,
	doSync,
	errorString,
	helpOpen,
	openPalette,
	palette,
	pushToast,
	refresh,
	searchQuery,
	selectedVobe,
	view,
	vobes,
} from "../lib/stores"
import type { Vobe } from "../lib/types"

type Cmd = {
	id: string
	label: string
	hint?: string
	run: () => void | Promise<void>
}

let inputEl: HTMLInputElement
let query = ""
let active = 0

$: isOpen = $palette.mode !== "closed"

const builtins: Cmd[] = [
	{
		id: "view.dashboard",
		label: "Go to Dashboard",
		hint: "1",
		run: () => view.set("dashboard"),
	},
	{
		id: "view.projects",
		label: "Go to Projects",
		hint: "2",
		run: () => view.set("projects"),
	},
	{
		id: "view.activity",
		label: "Go to Activity",
		hint: "3",
		run: () => view.set("activity"),
	},
	{
		id: "view.settings",
		label: "Go to Settings",
		hint: "4",
		run: () => view.set("settings"),
	},
	{
		id: "action.scan",
		label: "Scan for new vobes",
		hint: "⌘R",
		run: doScan,
	},
	{
		id: "action.sync",
		label: "Sync git state",
		hint: "⌘S",
		run: doSync,
	},
	{
		id: "action.refresh",
		label: "Refresh vobes from cache",
		run: () => refresh(),
	},
	{
		id: "action.add_folder",
		label: "Add a folder as a vobe…",
		run: () => addFolder(),
	},
	{
		id: "action.help",
		label: "Show keyboard shortcuts",
		hint: "?",
		run: () => helpOpen.set(true),
	},
	{
		id: "action.save_view",
		label: "Save current search as a view",
		run: () => {
			const q =
				(
					document.querySelector(
						".search input",
					) as HTMLInputElement | null
				)?.value ?? ""
			if (!q.trim()) {
				pushToast({ kind: "info", message: "Type a search first." })
				return
			}
			addSavedFilter(`View: ${q}`, q)
			pushToast({ kind: "success", message: `Saved view “${q}”.` })
		},
	},
	{
		id: "action.clear_search",
		label: "Clear search",
		run: () => {
			searchQuery.set("")
			pushToast({ kind: "info", message: "Search cleared." })
		},
	},
]

async function addFolder() {
	try {
		const picked = await openDialog({ directory: true, multiple: false })
		if (!picked) return
		const path = String(picked)
		const v = (await invoke("add_vobe", { path })) as Vobe
		await refresh()
		pushToast({ kind: "success", message: `Added ${v.name}.` })
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

$: projectResults = fuzzy(
	query,
	$vobes.map((v) => ({
		id: v.id,
		text: v.name,
		meta: [
			v.path,
			languageLabel(v.language),
			frameworkLabel(v.framework),
			v.git?.branch ?? "",
		]
			.filter(Boolean)
			.join(" · "),
		data: v,
	})),
	12,
)

$: commandResults = fuzzy(
	query,
	builtins.map((c) => ({ id: c.id, text: c.label, meta: c.hint ?? "" })),
	6,
)

$: items = [
	...projectResults.map((r) => ({ kind: "project" as const, ...r })),
	...commandResults.map((r) => {
		const cmd = builtins.find((b) => b.id === r.id)
		return { kind: "command" as const, ...r, run: cmd?.run ?? (() => {}) }
	}),
]

$: if (isOpen) reset()

function reset() {
	query = ""
	active = 0
	tick().then(() => inputEl?.focus())
}

function close() {
	closePalette()
}

function runItem(idx: number) {
	const it = items[idx]
	if (!it) return
	if (it.kind === "project") {
		const v = it.data as Vobe
		selectedVobe.set(v)
		view.set("projects")
		// Don't auto-open in editor — let the user press Enter on the detail view
		// to be deliberate. This is "jump to project", not "open project".
	} else {
		void it.run()
	}
	close()
}

function onKey(e: KeyboardEvent) {
	if (matchShortcut({ kind: "key", key: "escape" }, e)) {
		e.preventDefault()
		close()
		return
	}
	if (matchShortcut({ kind: "key", key: "enter" }, e)) {
		e.preventDefault()
		runItem(active)
		return
	}
	if (e.key === "ArrowDown" || (e.key === "n" && e.ctrlKey)) {
		e.preventDefault()
		active = Math.min(items.length - 1, active + 1)
		scrollIntoView()
		return
	}
	if (e.key === "ArrowUp" || (e.key === "p" && e.ctrlKey)) {
		e.preventDefault()
		active = Math.max(0, active - 1)
		scrollIntoView()
		return
	}
}

function scrollIntoView() {
	tick().then(() => {
		const el = document.querySelector(
			`.palette .item[data-idx="${active}"]`,
		) as HTMLElement | null
		el?.scrollIntoView({ block: "nearest" })
	})
}

function onGlobalKey(e: KeyboardEvent) {
	if (
		matchShortcut({ kind: "key", key: "k", mod: true }, e) ||
		matchShortcut({ kind: "key", key: "p", mod: true }, e)
	) {
		e.preventDefault()
		if (isOpen) close()
		else openPalette()
	}
}

onMount(() => {
	window.addEventListener("keydown", onGlobalKey)
	return () => window.removeEventListener("keydown", onGlobalKey)
})

function backdropClick(e: MouseEvent) {
	if (e.target === e.currentTarget) close()
}
</script>

{#if isOpen}
  <div
    class="backdrop"
    role="presentation"
    on:click={backdropClick}
  >
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
      class="palette"
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
      on:keydown={onKey}
    >
      <div class="input-row">
        <span class="prompt">{modKeyLabel()} K</span>
        <input
          bind:this={inputEl}
          bind:value={query}
          type="text"
          autocomplete="off"
          spellcheck="false"
          placeholder="Search vobes and commands…"
          aria-label="Command palette search"
        />
        <span class="esc">esc</span>
      </div>

      <div class="results">
        {#if items.length === 0}
          <div class="empty">No matches for “{query}”.</div>
        {:else}
          {#if projectResults.length > 0}
            <div class="group">Vobes</div>
            {#each projectResults as r, i (r.id)}
              {@const idx = i}
              <button
                type="button"
                class="item"
                class:active={active === idx}
                data-idx={idx}
                on:click={() => runItem(idx)}
                on:mouseenter={() => (active = idx)}
              >
                <span class="kind-dot git" title="Project"></span>
                <span class="text">
                  {#each r.text.split("") as ch, ci}
                    {#if r.matches.includes(ci)}
                      <span class="hit">{ch}</span>
                    {:else}
                      <span>{ch}</span>
                    {/if}
                  {/each}
                </span>
                <span class="meta">{r.meta ?? ""}</span>
              </button>
            {/each}
          {/if}
          {#if commandResults.length > 0}
            <div class="group">Commands</div>
            {#each commandResults as r, i (r.id)}
              {@const idx = projectResults.length + i}
              <button
                type="button"
                class="item cmd"
                class:active={active === idx}
                data-idx={idx}
                on:click={() => runItem(idx)}
                on:mouseenter={() => (active = idx)}
              >
                <span class="kind-dot cmd" title="Command">›</span>
                <span class="text">{r.text}</span>
                <span class="meta">{r.meta ?? ""}</span>
              </button>
            {/each}
          {/if}
        {/if}
      </div>

      <div class="foot">
        <span><kbd>↑</kbd><kbd>↓</kbd> navigate</span>
        <span><kbd>↵</kbd> select</span>
        <span><kbd>esc</kbd> close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 160;
    background: rgba(8, 10, 14, 0.45);
    backdrop-filter: blur(4px);
    display: flex; align-items: flex-start; justify-content: center;
    padding-top: 14vh;
    animation: fade 0.12s ease;
  }
  .palette {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 14px;
    width: min(640px, calc(100% - 32px));
    box-shadow: 0 30px 80px rgba(0,0,0,0.35);
    overflow: hidden;
    display: flex; flex-direction: column;
    max-height: 70vh;
    animation: pop 0.13s ease;
  }
  .input-row {
    display: flex; align-items: center; gap: 10px;
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
  }
  .input-row input {
    flex: 1;
    border: none; background: transparent; outline: none;
    font: inherit; color: var(--fg);
    padding: 4px 0;
  }
  .prompt {
    color: var(--fg-faint);
    font-size: 12px;
    font-family: ui-monospace, Menlo, monospace;
  }
  .esc {
    color: var(--fg-faint);
    font-size: 11px;
    background: var(--bg-sunken);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 6px;
  }
  .results { overflow-y: auto; padding: 6px; }
  .group {
    font-size: 11px; text-transform: uppercase; letter-spacing: 0.04em;
    color: var(--fg-faint); padding: 10px 10px 4px;
  }
  .item {
    width: 100%; text-align: left;
    display: grid;
    grid-template-columns: 18px 1fr auto;
    align-items: center; gap: 10px;
    padding: 7px 10px;
    border: none; background: transparent;
    border-radius: 8px;
    color: var(--fg);
    cursor: pointer;
    font-size: 13.5px;
  }
  .item.active { background: var(--accent-soft); color: var(--accent); }
  .item .kind-dot {
    width: 8px; height: 8px; border-radius: 50%;
    background: var(--fg-faint);
  }
  .item .kind-dot.git { background: var(--success); }
  .item .kind-dot.cmd {
    background: transparent; color: var(--fg-muted);
    width: auto; height: auto; font-weight: 700;
  }
  .item .text { font-weight: 500; }
  .item .meta { color: var(--fg-muted); font-size: 12px; }
  .item .hit { color: var(--accent); font-weight: 700; }
  .item.cmd .kind-dot.cmd { color: var(--accent); }
  .empty {
    padding: 24px; text-align: center;
    color: var(--fg-muted); font-size: 13px;
  }
  .foot {
    display: flex; gap: 14px;
    padding: 8px 14px;
    border-top: 1px solid var(--border);
    color: var(--fg-faint);
    font-size: 11px;
  }
  .foot kbd {
    font-family: ui-monospace, Menlo, monospace;
    border: 1px solid var(--border);
    background: var(--bg-sunken);
    padding: 0 5px;
    border-radius: 4px;
    margin-right: 4px;
  }
  @keyframes fade { from { opacity: 0; } }
  @keyframes pop { from { opacity: 0; transform: translateY(-6px) scale(0.98); } }
</style>
