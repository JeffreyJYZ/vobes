<script lang="ts">
import { open as openDialog } from "@tauri-apps/plugin-dialog"
import { onMount } from "svelte"
import {
	applyConfig,
	config,
	configPath,
	configPaths,
	errorString,
	loadConfig,
	pushToast,
	view,
} from "../lib/stores"
import type { Config } from "../lib/types"

let draft: Config | null = null
let dirty = false
let saving = false
let newExclude = ""

onMount(async () => {
	if (!$config) await loadConfig()
	draft = $config ? structuredClone($config) : null
})

$: if (draft && $config) {
	dirty = JSON.stringify(draft) !== JSON.stringify($config)
}

async function pickRoot() {
	try {
		const picked = await openDialog({ directory: true, multiple: false })
		if (picked && draft) {
			const p = String(picked)
			if (!draft.scan.roots.includes(p)) {
				draft.scan.roots = [...draft.scan.roots, p]
			}
		}
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

function removeRoot(r: string) {
	if (!draft) return
	draft.scan.roots = draft.scan.roots.filter((x) => x !== r)
}

function addExclude() {
	if (!draft) return
	const v = newExclude.trim()
	if (v && !draft.scan.exclude.includes(v)) {
		draft.scan.exclude = [...draft.scan.exclude, v]
	}
	newExclude = ""
}

function removeExclude(x: string) {
	if (!draft) return
	draft.scan.exclude = draft.scan.exclude.filter((t) => t !== x)
}

async function save() {
	if (!draft) return
	saving = true
	try {
		await applyConfig(draft)
		pushToast({ kind: "success", message: "Settings saved." })
	} finally {
		saving = false
	}
}

function onRootPathKeydown(e: KeyboardEvent) {
	if (e.key !== "Enter") return
	if (!draft) return
	const t = e.currentTarget as HTMLInputElement
	if (t.value && !draft.scan.roots.includes(t.value)) {
		draft.scan.roots = [...draft.scan.roots, t.value]
	}
	t.value = ""
}

function onExcludeKeydown(e: KeyboardEvent) {
	if (e.key !== "Enter") return
	addExclude()
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
        Folders Vobes walks to discover projects. Use <code>~</code> for your home directory.
      </p>
      <div class="list">
        {#each draft.scan.roots as r (r)}
          <div class="row-tag">
            <code>{r}</code>
            <button class="x" on:click={() => removeRoot(r)} aria-label="Remove">×</button>
          </div>
        {/each}
        {#if draft.scan.roots.length === 0}
          <div class="muted small">No roots — Vobes will find nothing.</div>
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
          <input type="number" min="1" max="10" bind:value={draft.scan.max_depth} />
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={draft.scan.follow_symlinks} />
          Follow symlinks
        </label>
      </div>
    </section>

    <section class="block">
      <h3>Excludes</h3>
      <p class="muted">Folder names to always skip (added on top of the built-in defaults).</p>
      <div class="list">
        {#each draft.scan.exclude as x (x)}
          <div class="row-tag">
            <span>{x}</span>
            <button class="x" on:click={() => removeExclude(x)} aria-label="Remove">×</button>
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
        <label>
          <span>Date format</span>
          <select bind:value={draft.display.date_format}>
            <option value="relative">Relative (3h ago)</option>
            <option value="absolute">Absolute (3:14pm)</option>
          </select>
        </label>
        <label>
          <span>Default sort</span>
          <select bind:value={draft.display.default_sort}>
            <option value="last_modified">Last modified</option>
            <option value="last_opened">Last opened</option>
            <option value="name">Name</option>
            <option value="created_at">Created</option>
          </select>
        </label>
      </div>
    </section>

    <section class="block">
      <h3>Git</h3>
      <div class="grid">
        <label>
          <span>Cache TTL (seconds)</span>
          <input
            type="number"
            min="0"
            step="5"
            bind:value={draft.git.cache_ttl_seconds}
          />
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={draft.git.fetch_upstream} />
          Fetch upstream on sync
        </label>
      </div>
    </section>

    <section class="block">
      <h3>Data locations</h3>
      <p class="muted small">
        Read-only.
        {#if $configPaths.state_dir.includes("-dev")}
          Dev build — uses a separate directory so it can't overwrite a release install.
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
        <label class="check">
          <input
            type="checkbox"
            bind:checked={draft.desktop.launch_on_login}
          />
          Launch Vobes on system login
        </label>
      </div>
    </section>

    <div class="actions">
      <button on:click={() => view.set("dashboard")}>Done</button>
      <button class="primary" on:click={save} disabled={!dirty || saving}>
        {saving ? "Saving…" : "Save changes"}
      </button>
    </div>
  {/if}
</div>

<style>
  .settings { max-width: 720px; margin: 0 auto; }
  .head {
    display: flex; align-items: center; justify-content: space-between;
    margin-bottom: 18px;
  }
  .head h2 { margin: 0; }
  .row { display: flex; align-items: center; }
  .row.gap { gap: 8px; margin-top: 10px; }
  .kv-paths {
    display: grid; grid-template-columns: 140px 1fr; gap: 8px 16px;
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
  .block h3 { margin: 0 0 4px; font-size: 14px; font-weight: 700; }
  .block .muted { margin: 0 0 10px; }
  .list {
    display: flex; flex-wrap: wrap; gap: 6px;
    margin: 8px 0;
  }
  .row-tag {
    display: flex; align-items: center; gap: 6px;
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
    background: transparent; border: none;
    color: var(--fg-muted); cursor: pointer;
    font-size: 15px; line-height: 1; padding: 0 2px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 10px 16px;
  }
  .grid label {
    display: flex; flex-direction: column; gap: 4px;
    font-size: 12.5px; color: var(--fg-muted);
  }
  .grid label.check {
    flex-direction: row; align-items: center; gap: 6px;
    color: var(--fg);
  }
  .grid input, .grid select {
    padding: 6px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--fg);
    font: inherit;
  }
  .grid input:focus, .grid select:focus {
    outline: 2px solid var(--accent-soft); border-color: var(--accent);
  }
  .actions {
    display: flex; justify-content: flex-end; gap: 8px;
    margin-top: 12px;
  }
  .small { font-size: 12px; }
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
    outline: 2px solid var(--accent-soft); border-color: var(--accent);
  }
</style>
