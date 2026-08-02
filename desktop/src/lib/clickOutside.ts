// Svelte action: fire `callback` when a pointerdown lands outside `node`.
// Used by popovers/dropdowns to close on outside click.

import type { Action } from "svelte/action"

export const clickOutside: Action<HTMLElement, () => void> = (
	node,
	callback,
) => {
	function handler(e: PointerEvent) {
		if (!node.contains(e.target as Node)) {
			callback()
		}
	}
	// Capture phase so we run before any "stopPropagation" inside the node.
	document.addEventListener("pointerdown", handler, true)
	return {
		update(next: () => void) {
			callback = next
		},
		destroy() {
			document.removeEventListener("pointerdown", handler, true)
		},
	}
}
