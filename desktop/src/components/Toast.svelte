<script lang="ts">
	import { dismissToast, toasts } from "../lib/stores";
</script>

<div class="toast-stack" aria-live="polite" aria-atomic="false">
	{#each $toasts as t (t.id)}
		{#if t.action}
			<div class="toast {t.kind}" role="group">
				<span class="dot"></span>
				<span class="msg">{t.message}</span>
				<button
					type="button"
					class="act"
					on:click={async () => {
						const action = t.action;
						dismissToast(t.id);
						if (action) await action.run();
					}}
				>{t.action.label}</button>
			</div>
		{:else}
			<div
				class="toast {t.kind}"
				role="button"
				tabindex="0"
				aria-label="Dismiss notification"
				on:click={() => dismissToast(t.id)}
				on:keydown={(e) => {
					if (e.key === "Enter" || e.key === " ") {
						e.preventDefault();
						dismissToast(t.id);
					}
				}}
			>
				<span class="dot"></span>
				<span class="msg">{t.message}</span>
				{#if t.kind === "error" || t.ttl === 0}
					<button
						type="button"
						class="close"
						aria-label="Close"
						on:click|stopPropagation={() => dismissToast(t.id)}
					>×</button>
				{/if}
			</div>
		{/if}
	{/each}
</div>

<style>
	.toast-stack {
		position: fixed;
		right: 18px;
		bottom: 18px;
		z-index: 200;
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-width: 380px;
		pointer-events: none;
	}
	.toast {
		pointer-events: auto;
		display: flex;
		align-items: flex-start;
		gap: 10px;
		text-align: left;
		padding: 10px 14px;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--bg-elevated);
		color: var(--fg);
		box-shadow: var(--shadow-md);
		font-size: 13px;
		line-height: 1.4;
		cursor: pointer;
		animation: slidein 0.18s ease;
	}
	.toast .dot {
		flex: none;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		margin-top: 6px;
		background: var(--fg-faint);
	}
	.toast.success .dot {
		background: var(--success);
	}
	.toast.error .dot {
		background: var(--danger);
	}
	.toast.info .dot {
		background: var(--accent);
	}
	.toast .msg {
		white-space: pre-wrap;
		flex: 1;
	}
	.toast .close {
		flex: none;
		border: none;
		background: transparent;
		font: inherit;
		font-size: 16px;
		line-height: 1;
		color: var(--fg-muted);
		padding: 2px 0 0 6px;
		cursor: pointer;
	}
	.toast .act {
		flex: none;
		border: 1px solid var(--accent);
		background: transparent;
		color: var(--accent);
		font: inherit;
		font-size: 12px;
		font-weight: 500;
		padding: 4px 10px;
		border-radius: 6px;
		cursor: pointer;
	}
	.toast .act:hover {
		background: var(--accent);
		color: var(--bg);
	}
	.toast .close:hover {
		color: var(--fg);
	}
	@keyframes slidein {
		from {
			transform: translateY(8px);
			opacity: 0;
		}
	}
</style>
