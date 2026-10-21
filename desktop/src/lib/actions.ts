import { ask, open as openDialog } from "@tauri-apps/plugin-dialog"
import { get } from "svelte/store"
import * as api from "./api"
import {
	addSavedFilter,
	errorString,
	missingVobes,
	pushToast,
	refresh,
	searchQuery,
	selectedVobe,
} from "./stores"
import type { Vobe } from "./types"

/// Pick one directory; toasts on dialog failure. Null when cancelled.
export async function pickDir(): Promise<string | null> {
	try {
		const picked = await openDialog({ directory: true, multiple: false })
		return picked ? String(picked) : null
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
		return null
	}
}

/// Open the vobe's directory in the default editor. Resolves the user's
/// stored default from localStorage via the Projects view's local
/// handlers; this is a thin pass-through. A failure rejects, so the
/// caller owns the message — swallowing it here produced an error toast
/// *and* a success toast for one failed click.
export async function openInEditor(v: Vobe, app?: string) {
	await api.markOpened(v.name)
	await api.openInEditor(v.name, app)
}

/// Spawn the default terminal at the vobe's path.
export async function openInTerminal(v: Vobe, app?: string) {
	await api.markOpened(v.name)
	await api.openInTerminal(v.name, app)
}

/// Reveal in the OS file manager.
export async function revealInFinder(v: Vobe) {
	await api.revealInFinder(v.name)
}

/// Copy the absolute path to the clipboard.
export async function copyPath(v: Vobe) {
	try {
		await api.copyText(v.path)
		pushToast({ kind: "info", message: "Path copied." })
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

/// Toggle the pinned flag and refresh so the dashboard re-sorts.
/// Also nudges `selectedVobe` if it points at the same vobe, so the
/// Projects-view Pin/Unpin button label flips without a navigation.
export async function togglePin(v: Vobe) {
	try {
		await api.setPinned(v.id, !v.pinned)
		await refresh({ silent: true })
		const cur = get(selectedVobe)
		if (cur && cur.id === v.id) {
			selectedVobe.set({ ...cur, pinned: !v.pinned })
		}
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

/// Confirm then untrack. Files on disk untouched.
export async function removeVobe(v: Vobe) {
	const ok = await ask(
		`Remove ${v.name}? This only untracks it; the files are untouched.`,
		{ title: "Vobes", kind: "warning" },
	)
	if (!ok) return
	try {
		await api.removeVobe(v.name)
		await refresh({ silent: true })
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

/// Delete the project directory from disk, then stop tracking it.
/// Two confirmations: this is a recursive delete with no Trash and no
/// undo, and the path is what's being destroyed.
export async function deleteProjectFromDisk(v: Vobe) {
	// The dialog calls sit inside the try: a failed `ask` used to reject
	// before any handler was attached, which reached the global
	// unhandled-rejection handler as a red strip over the app.
	try {
		const ok = await ask(
			`Delete ${v.path} from disk? Every file in that directory is removed. This cannot be undone.`,
			{ title: "Delete project", kind: "warning" },
		)
		if (!ok) return
		const sure = await ask(
			`Last check: permanently delete ${v.name}? There is no Trash and no undo.`,
			{ title: "Delete project", kind: "warning" },
		)
		if (!sure) return
		await api.deleteProjectDir(v.name)
		if (get(selectedVobe)?.id === v.id) selectedVobe.set(null)
		await refresh({ silent: true })
		pushToast({ kind: "success", message: `Deleted ${v.path}.` })
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

/// Delete a project's `.git` directory. Files stay, history does not.
export async function deleteGitDir(v: Vobe) {
	try {
		const ok = await ask(
			`Delete ${v.path}/.git? The commit history is gone; the working files stay. This cannot be undone.`,
			{ title: "Delete .git", kind: "warning" },
		)
		if (!ok) return
		const fresh = await api.deleteGitDir(v.name)
		if (get(selectedVobe)?.id === v.id) selectedVobe.set(fresh)
		await refresh({ silent: true })
		pushToast({ kind: "success", message: "Removed .git." })
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

/// Untrack a vobe whose tracked path is gone. No confirm: the files are
/// already gone, so this only drops a dead row.
export async function untrackMissing(v: Vobe) {
	try {
		await api.removeVobe(v.name)
		if (get(selectedVobe)?.id === v.id) selectedVobe.set(null)
		await refresh({ silent: true })
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}

/// Pin the current search as a saved view. One implementation, used by
/// both the Dashboard button and the command palette — the palette used
/// to scrape the Dashboard's DOM, so it only worked on that view.
export async function saveCurrentSearchAsView(): Promise<void> {
	const q = get(searchQuery).trim()
	if (!q) {
		pushToast({ kind: "info", message: "Type a search first." })
		return
	}
	try {
		await addSavedFilter(`View: ${q}`, q)
		pushToast({ kind: "success", message: `Saved view “${q}”.` })
	} catch {
		pushToast({ kind: "error", message: "Could not save view." })
	}
}

/// Confirm then untrack every vobe whose path is gone. Guarded because
/// an unmounted volume reads as "missing" for all of its projects.
export async function purgeMissing() {
	const n = get(missingVobes).length
	const ok = await ask(
		`Untrack ${n} vobe${n === 1 ? "" : "s"} whose path no longer exists? Files on disk are untouched — but an unmounted drive also reads as missing.`,
		{ title: "Vobes", kind: "warning" },
	)
	if (!ok) return
	try {
		const gone = await api.purgeMissing()
		const cur = get(selectedVobe)
		if (cur && get(missingVobes).some((v) => v.id === cur.id)) {
			selectedVobe.set(null)
		}
		await refresh({ silent: true })
		pushToast({
			kind: "success",
			message: `Untracked ${gone} missing vobe${gone === 1 ? "" : "s"}.`,
		})
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	}
}
