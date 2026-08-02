<script lang="ts">
// Generic dropdown — a button that opens a popover of options.
// Replaces native `<select>` everywhere, because on macOS the OS
// ignores most of the CSS we throw at the native control, and the
// look ends up inconsistent with the rest of the app.

import { clickOutside } from "../lib/clickOutside"

type Option<V extends string = string> = { value: V; label: string }

export let value: string
export let options: Option[]
export let ariaLabel: string = ""
export let onChange: (v: string) => void = () => {}
export let width: string = "auto"

let open = false
let triggerEl: HTMLButtonElement

function pick(v: string) {
	onChange(v)
	open = false
	triggerEl?.focus()
}

function onTriggerKey(e: KeyboardEvent) {
	if (e.key === "ArrowDown" || e.key === "Enter" || e.key === " ") {
		e.preventDefault()
		open = true
	} else if (e.key === "Escape" && open) {
		e.preventDefault()
		open = false
	}
}

function onMenuKey(e: KeyboardEvent) {
	if (e.key === "Escape") {
		e.preventDefault()
		open = false
		triggerEl?.focus()
	}
}

$: current = options.find((o) => o.value === value)
</script>

<div class="select" style="width: {width}" use:clickOutside={() => (open = false)}>
  <button
    type="button"
    class="trigger"
    class:open
    bind:this={triggerEl}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={ariaLabel}
    on:click={() => (open = !open)}
    on:keydown={onTriggerKey}
  >
    <span class="label">{current?.label ?? value}</span>
    <span class="chev" aria-hidden="true">▾</span>
  </button>
  {#if open}
    <ul class="menu" role="listbox" aria-label={ariaLabel} on:keydown={onMenuKey}>
      {#each options as o (o.value)}
        <li>
          <button
            type="button"
            class="opt"
            class:active={o.value === value}
            role="option"
            aria-selected={o.value === value}
            on:click={() => pick(o.value)}
          >
            {o.label}
            {#if o.value === value}<span class="check" aria-hidden="true">✓</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .select {
    position: relative;
    display: inline-block;
  }
  .trigger {
    display: flex; align-items: center; justify-content: space-between; gap: 8px;
    width: 100%;
    padding: 7px 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--fg);
    font: inherit;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.12s ease;
  }
  .trigger:hover { border-color: var(--fg-faint); }
  .trigger.open {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }
  .trigger .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .trigger .chev {
    color: var(--fg-muted);
    font-size: 12px;
    font-weight: 700;
    transition: transform 0.15s ease;
  }
  .trigger.open .chev { transform: rotate(180deg); }
  .menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 60;
    margin: 0; padding: 4px;
    list-style: none;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: var(--shadow-md);
    min-width: 100%;
    animation: pop 0.1s ease;
  }
  .menu li { margin: 0; padding: 0; }
  .opt {
    display: flex; align-items: center; justify-content: space-between;
    width: 100%;
    padding: 7px 10px;
    border: none; background: transparent;
    border-radius: 5px;
    color: var(--fg);
    font: inherit; font-size: 13px;
    cursor: pointer;
    text-align: left;
  }
  .opt:hover { background: var(--bg-sunken); }
  .opt.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }
  .opt .check {
    color: var(--accent);
    font-weight: 700;
  }
  @keyframes pop {
    from { opacity: 0; transform: translateY(-4px); }
  }
</style>
