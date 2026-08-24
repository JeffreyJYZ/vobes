import { ask } from "@tauri-apps/plugin-dialog";
import * as api from "./api";
import { errorString, pushToast, refresh } from "./stores";
import type { Vobe } from "./types";

/// Open the vobe's directory in the default editor. Resolves the user's
/// stored default from localStorage via the Projects view's local
/// handlers; this is a thin pass-through that toasts on failure.
export async function openInEditor(v: Vobe, app?: string) {
	try {
		await api.markOpened(v.name);
		await api.openInEditor(v.name, app);
	} catch (e) {
		pushToast({ kind: "error", message: `Editor: ${errorString(e)}` });
	}
}

/// Spawn the default terminal at the vobe's path.
export async function openInTerminal(v: Vobe, app?: string) {
	try {
		await api.markOpened(v.name);
		await api.openInTerminal(v.name, app);
	} catch (e) {
		pushToast({ kind: "error", message: `Terminal: ${errorString(e)}` });
	}
}

/// Reveal in the OS file manager.
export async function revealInFinder(v: Vobe) {
	try {
		await api.revealInFinder(v.name);
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) });
	}
}

/// Copy the absolute path to the clipboard.
export async function copyPath(v: Vobe) {
	try {
		await api.copyText(v.path);
		pushToast({ kind: "info", message: "Path copied." });
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) });
	}
}

/// Toggle the pinned flag and refresh so the dashboard re-sorts.
export async function togglePin(v: Vobe) {
	try {
		await api.setPinned(v.name, !v.pinned);
		await refresh({ silent: true });
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) });
	}
}

/// Confirm then untrack. Files on disk untouched.
export async function removeVobe(v: Vobe) {
	const ok = await ask(
		`Remove ${v.name}? This only untracks it; the files are untouched.`,
		{ title: "Vobes", kind: "warning" },
	);
	if (!ok) return;
	try {
		await api.removeVobe(v.name);
		await refresh({ silent: true });
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) });
	}
}