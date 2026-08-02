// Centralized shortcut definitions + matching. Components import a
// single `matchShortcut` helper instead of hand-rolling per-component
// key checks, so behavior stays consistent across the app.

import { isMac } from "./format"

export type Shortcut = {
	id: string
	description: string
	/** Pretty label for the cheatsheet (e.g. "⌘ K"). */
	label: string
	/** Key combo to match against incoming `KeyboardEvent`s. */
	combo: ShortcutCombo
	/** If true, fires even when focus is inside a text input. Default false. */
	allowInInput?: boolean
}

export type ShortcutCombo =
	| {
			kind: "key"
			key: string
			mod?: boolean
			shift?: boolean
			alt?: boolean
	  }
	| {
			kind: "letter"
			letter: string
			mod?: boolean
			shift?: boolean
			alt?: boolean
	  }

const META = isMac()

function comboToEvent(combo: ShortcutCombo): {
	key: string
	mod: boolean
	shift: boolean
	alt: boolean
} {
	if (combo.kind === "key") {
		return {
			key: combo.key.toLowerCase(),
			mod: !!combo.mod,
			shift: !!combo.shift,
			alt: !!combo.alt,
		}
	}
	return {
		key: combo.letter.toLowerCase(),
		mod: !!combo.mod,
		shift: !!combo.shift,
		alt: !!combo.alt,
	}
}

export function matchShortcut(combo: ShortcutCombo, e: KeyboardEvent): boolean {
	const want = comboToEvent(combo)
	const haveMod = META ? e.metaKey : e.ctrlKey
	if (haveMod !== want.mod) return false
	if (e.shiftKey !== want.shift) return false
	if (e.altKey !== want.alt) return false
	return e.key.toLowerCase() === want.key
}

export function comboLabel(combo: ShortcutCombo): string {
	const parts: string[] = []
	const want = comboToEvent(combo)
	if (want.mod) parts.push(META ? "⌘" : "Ctrl")
	if (want.shift) parts.push("⇧")
	if (want.alt) parts.push(META ? "⌥" : "Alt")
	parts.push(combo.kind === "key" ? combo.key : combo.letter.toUpperCase())
	return parts.join(META ? "" : "+")
}

/** Build a `Shortcut` from a combo + label parts. */
export function defineShortcut(
	id: string,
	description: string,
	combo: ShortcutCombo,
	opts: { allowInInput?: boolean } = {},
): Shortcut {
	return {
		id,
		description,
		label: comboLabel(combo),
		combo,
		allowInInput: opts.allowInInput,
	}
}

// All app shortcuts in one place. Source of truth for the help overlay.
export const shortcuts: Shortcut[] = [
	defineShortcut("palette", "Open command palette", {
		kind: "key",
		key: "k",
		mod: true,
	}),
	defineShortcut("help", "Show keyboard shortcuts", {
		kind: "key",
		key: "/",
		shift: true,
	}),
	defineShortcut("search", "Focus search", { kind: "key", key: "/" }),
	defineShortcut("dashboard", "Go to dashboard", { kind: "key", key: "1" }),
	defineShortcut("projects", "Go to projects", { kind: "key", key: "2" }),
	defineShortcut("activity", "Go to activity", { kind: "key", key: "3" }),
	defineShortcut("settings", "Go to settings", { kind: "key", key: "4" }),
	defineShortcut("scan", "Scan for new vobes", {
		kind: "key",
		key: "r",
		mod: true,
	}),
	defineShortcut("sync", "Sync git state", {
		kind: "key",
		key: "s",
		mod: true,
	}),
	defineShortcut("up", "Move up", { kind: "key", key: "arrowup" }),
	defineShortcut("down", "Move down", { kind: "key", key: "arrowdown" }),
	defineShortcut("enter", "Open / confirm", { kind: "key", key: "enter" }),
	defineShortcut("escape", "Close / cancel", { kind: "key", key: "escape" }),
]

export function isTypingTarget(e: KeyboardEvent): boolean {
	const t = e.target as HTMLElement | null
	if (!t) return false
	const tag = t.tagName
	if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true
	if (t.isContentEditable) return true
	return false
}
