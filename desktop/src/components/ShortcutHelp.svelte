<script lang="ts">
import { isMac } from "../lib/format"
import { shortcuts } from "../lib/keyboard"
import { helpOpen } from "../lib/stores"

function close() {
	helpOpen.set(false)
}
function backdrop(e: MouseEvent) {
	if (e.target === e.currentTarget) close()
}
</script>

{#if $helpOpen}
  <div
    class="backdrop"
    role="presentation"
    on:click={backdrop}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-label="Keyboard shortcuts"
    >
      <div class="head">
        <h3>Keyboard shortcuts</h3>
        <button class="close" type="button" on:click={close} aria-label="Close">×</button>
      </div>
      <div class="hint">
        Tip: press <kbd>?</kbd> any time to bring this up.
        {#if isMac()}Use <kbd>⌘</kbd> as the modifier.{:else}Use <kbd>Ctrl</kbd> as the modifier.{/if}
      </div>
      <ul>
        {#each shortcuts as s (s.id)}
          <li>
            <span class="desc">{s.description}</span>
            <span class="combo"><kbd>{s.label}</kbd></span>
          </li>
        {/each}
      </ul>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 150;
    background: rgba(8, 10, 14, 0.5);
    backdrop-filter: blur(3px);
    display: flex; align-items: center; justify-content: center;
    animation: fade 0.15s ease;
  }
  .modal {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 22px 26px;
    width: min(520px, calc(100% - 40px));
    box-shadow: 0 24px 60px rgba(0,0,0,0.3);
    animation: pop 0.16s ease;
  }
  .head { display: flex; align-items: center; justify-content: space-between; }
  .head h3 { margin: 0; font-size: 17px; font-weight: 700; }
  .close {
    background: transparent; border: none; color: var(--fg-muted);
    font-size: 22px; padding: 0 6px; cursor: pointer; line-height: 1;
  }
  .hint {
    margin: 8px 0 16px;
    color: var(--fg-muted);
    font-size: 12.5px;
  }
  ul {
    list-style: none; margin: 0; padding: 0;
    display: grid; grid-template-columns: 1fr 1fr; gap: 6px 24px;
  }
  li {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 0; border-bottom: 1px solid var(--border);
    font-size: 13px;
  }
  li:nth-last-child(-n+2) { border-bottom: none; }
  .desc { color: var(--fg); }
  .combo kbd {
    font-family: ui-monospace, Menlo, monospace;
    background: var(--bg-sunken);
    color: var(--fg-muted);
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 1px 7px;
    font-size: 11.5px;
  }
  @keyframes fade { from { opacity: 0; } }
  @keyframes pop { from { opacity: 0; transform: scale(0.97); } }
</style>
