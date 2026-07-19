<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type Vobe = {
    id: string;
    name: string;
    path: string;
    language: string | null;
    framework: string | null;
    package_manager: string | null;
    last_modified: string | null;
    git: {
      branch: string;
      dirty: boolean;
      ahead: number;
      behind: number;
    } | null;
    tags: string[];
  };

  type ActivityEvent = {
    id: number | null;
    vobe_id: string;
    kind: string;
    timestamp: string;
    detail: string | null;
  };

  let view: "dashboard" | "projects" | "activity" = "dashboard";
  let vobes: Vobe[] = [];
  let activity: ActivityEvent[] = [];
  let loading = false;
  let error: string | null = null;
  let selected: Vobe | null = null;
  let confirmReset = false;

  onMount(async () => {
    await refresh();
  });

  async function refresh() {
    loading = true;
    error = null;
    try {
      [vobes, activity] = await Promise.all([
        invoke<Vobe[]>("list_vobes"),
        invoke<ActivityEvent[]>("recent_activity", { limit: 50 }),
      ]);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function scan() {
    loading = true;
    try {
      await invoke("scan");
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function resetAndRescan() {
    loading = true;
    error = null;
    try {
      const found = await invoke<number>("reset_and_rescan");
      await refresh();
      confirmReset = false;
      error = `Reset complete — ${found} vobes re-discovered.`;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function relative(ts: string | null): string {
    if (!ts) return "-";
    const d = new Date(ts);
    const s = (Date.now() - d.getTime()) / 1000;
    if (s < 60) return "just now";
    if (s < 3600) return `${Math.floor(s / 60)}m ago`;
    if (s < 86400) return `${Math.floor(s / 3600)}h ago`;
    if (s < 2592000) return `${Math.floor(s / 86400)}d ago`;
    return `${Math.floor(s / 2592000)}mo ago`;
  }

  function vobeName(id: string): string {
    return vobes.find((v) => v.id === id)?.name ?? id.slice(0, 10);
  }

  function setView(next: "dashboard" | "projects" | "activity") {
    view = next;
    selected = null;
  }
</script>

<div class="app">
  <aside class="sidebar">
    <div class="brand">
      <svg class="logo" viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M3 4 L12 20 L21 4"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <h1>Vobes</h1>
    </div>
    <button
      type="button"
      class="nav-item"
      class:active={view === "dashboard"}
      on:click={() => setView("dashboard")}
    >
      Dashboard
    </button>
    <button
      type="button"
      class="nav-item"
      class:active={view === "projects"}
      on:click={() => setView("projects")}
    >
      Projects
    </button>
    <button
      type="button"
      class="nav-item"
      class:active={view === "activity"}
      on:click={() => setView("activity")}
    >
      Activity
    </button>
  </aside>

  <main class="main">
    {#if error}
      <div class="error" style="color: var(--danger); margin-bottom: 16px;">
        {error}
      </div>
    {/if}

    {#if view === "dashboard"}
      <h2>Dashboard</h2>
      <div class="toolbar">
        <span>{vobes.length} vobes · {activity.length} recent events</span>
        <button class="primary" on:click={scan} disabled={loading}>
          {loading ? "Scanning…" : "Scan"}
        </button>
      </div>
      {#if vobes.length === 0}
        <div class="empty">
          <strong>No vobes yet</strong>
          Click <em>Scan</em> to discover projects in your configured roots.
        </div>
      {:else}
        <div class="vobe-grid">
          {#each vobes as v (v.id)}
            <button
              type="button"
              class="vobe-card"
              on:click={() => {
                selected = v;
                view = "projects";
              }}
            >
              <div class="name">{v.name}</div>
              <div class="meta">
                <span>{v.language ?? "-"}</span>
                <span>·</span>
                <span>{v.package_manager ?? "-"}</span>
              </div>
              <div class="status">
                {#if v.git}
                  <span class="badge">{v.git.branch}</span>
                  {#if v.git.dirty}
                    <span class="badge dirty">dirty</span>
                  {/if}
                  {#if v.git.ahead > 0}
                    <span class="badge ahead">↑{v.git.ahead}</span>
                  {/if}
                  {#if v.git.behind > 0}
                    <span class="badge behind">↓{v.git.behind}</span>
                  {/if}
                {/if}
                <span class="badge">{relative(v.last_modified)}</span>
              </div>
              <div class="path">{v.path}</div>
            </button>
          {/each}
        </div>
      {/if}
    {:else if view === "projects"}
      <div class="view-head">
        <h2>{selected ? selected.name : "Projects"}</h2>
        <div class="view-actions">
          <button class="primary" on:click={scan} disabled={loading}>
            {loading ? "Scanning…" : "Rescan"}
          </button>
          <button
            class="danger"
            on:click={() => (confirmReset = true)}
            disabled={loading}
          >
            Reset &amp; Rescan
          </button>
        </div>
      </div>
      {#if selected}
        <button on:click={() => (selected = null)}>← Back</button>
        <div class="detail-card">
          <div class="detail-grid">
            <div class="k">Path</div>
            <div class="v" style="font-family: ui-monospace, Menlo, monospace; font-size: 12px;">{selected.path}</div>
            <div class="k">Language</div>
            <div class="v">{selected.language ?? "-"}</div>
            <div class="k">Framework</div>
            <div class="v">{selected.framework ?? "-"}</div>
            <div class="k">Package manager</div>
            <div class="v">{selected.package_manager ?? "-"}</div>
            <div class="k">Tags</div>
            <div class="v">{selected.tags.join(", ") || "-"}</div>
          </div>
          {#if selected.git}
            <div class="section-title">Git</div>
            <div class="detail-grid">
              <div class="k">Branch</div>
              <div class="v">{selected.git.branch}</div>
              <div class="k">Status</div>
              <div class="v">
                {selected.git.dirty ? "dirty" : "clean"}
                {#if selected.git.ahead > 0} · ↑{selected.git.ahead}{/if}
                {#if selected.git.behind > 0} · ↓{selected.git.behind}{/if}
              </div>
            </div>
          {/if}
        </div>
        <button
          class="primary"
          style="margin-top: 16px;"
          on:click={async () => {
            if (selected) await invoke("open_vobe", { name: selected.name });
          }}
        >
          Open
        </button>
      {:else}
        <div class="vobe-grid">
          {#each vobes as v (v.id)}
            <button type="button" class="vobe-card" on:click={() => (selected = v)}>
              <div class="name">{v.name}</div>
              <div class="meta">
                <span>{v.language ?? "-"}</span>
                <span>·</span>
                <span>{v.package_manager ?? "-"}</span>
              </div>
              <div class="path">{v.path}</div>
            </button>
          {/each}
        </div>
      {/if}
    {:else if view === "activity"}
      <div class="view-head">
        <h2>Activity</h2>
        <button on:click={refresh} disabled={loading}>Refresh</button>
      </div>
      <table style="width: 100%; border-collapse: collapse;">
        <thead>
          <tr>
            <th style="text-align:left; padding: 6px;">When</th>
            <th style="text-align:left; padding: 6px;">Vobe</th>
            <th style="text-align:left; padding: 6px;">Kind</th>
            <th style="text-align:left; padding: 6px;">Detail</th>
          </tr>
        </thead>
        <tbody>
          {#each activity as e, i (i)}
            <tr style="border-top: 1px solid var(--border);">
              <td style="padding: 6px;">{relative(e.timestamp)}</td>
              <td style="padding: 6px;">{vobeName(e.vobe_id)}</td>
              <td style="padding: 6px;">{e.kind}</td>
              <td style="padding: 6px; color: var(--fg-muted);">{e.detail ?? ""}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </main>

  {#if confirmReset}
    <button
      class="modal-backdrop"
      type="button"
      aria-label="Close dialog"
      on:click={(e) => {
        if (e.target === e.currentTarget) confirmReset = false;
      }}
    >
      <div
        class="modal"
        role="dialog"
        aria-modal="true"
        aria-label="Reset and rescan confirmation"
        tabindex="-1"
      >
        <h3>Reset &amp; Rescan?</h3>
        <p>
          This will <strong>delete every vobe and all activity</strong>. There
          is no undo. The app will then re-scan all roots from scratch.
        </p>
        <div class="modal-actions">
          <button on:click={() => (confirmReset = false)} disabled={loading}>
            Cancel
          </button>
          <button class="danger" on:click={resetAndRescan} disabled={loading}>
            {loading ? "Working…" : "Delete everything & rescan"}
          </button>
        </div>
      </div>
    </button>
  {/if}
</div>