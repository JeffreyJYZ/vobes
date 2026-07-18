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
</script>

<div class="app">
  <aside class="sidebar">
    <h1>Vobes</h1>
    <div
      class="nav-item"
      class:active={view === "dashboard"}
      on:click={() => {
        view = "dashboard";
        selected = null;
      }}
    >
      Dashboard
    </div>
    <div
      class="nav-item"
      class:active={view === "projects"}
      on:click={() => {
        view = "projects";
        selected = null;
      }}
    >
      Projects
    </div>
    <div
      class="nav-item"
      class:active={view === "activity"}
      on:click={() => {
        view = "activity";
        selected = null;
      }}
    >
      Activity
    </div>
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
        <div class="empty">No vobes yet. Click Scan to discover projects.</div>
      {:else}
        <div class="vobe-grid">
          {#each vobes as v (v.id)}
            <div
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
            </div>
          {/each}
        </div>
      {/if}
    {:else if view === "projects"}
      <h2>{selected ? selected.name : "Projects"}</h2>
      {#if selected}
        <button on:click={() => (selected = null)}>← Back</button>
        <div style="margin-top: 16px;">
          <div><strong>Path:</strong> {selected.path}</div>
          <div><strong>Language:</strong> {selected.language ?? "-"}</div>
          <div><strong>Framework:</strong> {selected.framework ?? "-"}</div>
          <div>
            <strong>Package manager:</strong>
            {selected.package_manager ?? "-"}
          </div>
          <div><strong>Tags:</strong> {selected.tags.join(", ") || "-"}</div>
          {#if selected.git}
            <h3 style="margin-top: 16px;">Git</h3>
            <div>Branch: {selected.git.branch}</div>
            <div>Dirty: {selected.git.dirty}</div>
            <div>Ahead/behind: ↑{selected.git.ahead} ↓{selected.git.behind}</div>
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
            <div class="vobe-card" on:click={() => (selected = v)}>
              <div class="name">{v.name}</div>
              <div class="meta">
                <span>{v.language ?? "-"}</span>
                <span>·</span>
                <span>{v.package_manager ?? "-"}</span>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {:else if view === "activity"}
      <h2>Activity</h2>
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
          {#each activity as e (e.id ?? e.timestamp)}
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
</div>