// Thin wrappers around the Tauri `invoke` API so call sites stay tidy
// and any future renaming happens in one place.

import { invoke } from "@tauri-apps/api/core"
import {
	open as openExternal,
	Command as ShellCommand,
} from "@tauri-apps/plugin-shell"
import type {
	ActivityEvent,
	Config,
	ConfigDto,
	ContextPack,
	EditorApp,
	SavedFilterDto,
	SnapshotInfo,
	TerminalApp,
	TodoHit,
	Vobe,
} from "./types"

export async function listVobes(): Promise<Vobe[]> {
	return invoke<Vobe[]>("list_vobes")
}

export async function getVobe(name: string): Promise<Vobe | null> {
	return invoke<Vobe | null>("get_vobe", { name })
}

export async function recentActivity(
	limit: number,
	actor?: string,
): Promise<ActivityEvent[]> {
	return invoke<ActivityEvent[]>("recent_activity", {
		limit,
		actor: actor ?? null,
	})
}

export async function vobeActivity(
	vobeId: string,
	limit: number,
): Promise<ActivityEvent[]> {
	return invoke<ActivityEvent[]>("vobe_activity", {
		vobeId,
		limit,
	})
}

export async function scan(): Promise<number> {
	return invoke<number>("scan")
}

export async function sync(): Promise<[number, number]> {
	return invoke<[number, number]>("sync")
}

export async function resetAndRescan(): Promise<number> {
	return invoke<number>("reset_and_rescan")
}

export async function removeVobe(name: string): Promise<void> {
	return invoke<void>("remove_vobe", { name })
}

export async function addVobe(path: string): Promise<Vobe> {
	return invoke<Vobe>("add_vobe", { path })
}

export async function markOpened(name: string): Promise<void> {
	return invoke<void>("open_vobe", { name })
}

export async function exportJson(): Promise<string> {
	return invoke<string>("export_json", { out: null })
}

export async function getConfig(): Promise<ConfigDto> {
	return invoke<ConfigDto>("get_config")
}

export async function saveConfig(config: Config): Promise<ConfigDto> {
	return invoke<ConfigDto>("save_config", { newConfig: config })
}

export async function openInTerminal(name: string): Promise<void> {
	return invoke<void>("open_in_terminal", { name })
}

export async function openInTerminalWith(name: string, app: string): Promise<void> {
	return invoke<void>("open_in_terminal", { name, app })
}

export async function listTerminals(): Promise<TerminalApp[]> {
	return invoke<TerminalApp[]>("list_terminals")
}

export async function listEditors(): Promise<EditorApp[]> {
	return invoke<EditorApp[]>("list_editors")
}

export async function openInEditor(name: string, app?: string): Promise<void> {
	return invoke<void>("open_in_editor", { name, app: app ?? null })
}

export async function revealInFinder(name: string): Promise<void> {
	return invoke<void>("reveal_in_finder", { name })
}

export async function saveNotes(
	name: string,
	notes: string | null,
): Promise<Vobe> {
	return invoke<Vobe>("save_notes", { name, notes })
}

export async function setPinned(name: string, pinned: boolean): Promise<void> {
	return invoke<void>("set_pinned", { name, pinned })
}

export async function getPinned(): Promise<string[]> {
	return invoke<string[]>("get_pinned")
}

export async function setTags(name: string, tags: string[]): Promise<Vobe> {
	return invoke<Vobe>("set_tags", { name, tags })
}

export async function readReadme(name: string): Promise<string | null> {
	return invoke<string | null>("read_readme", { name })
}

export async function scrapeTodos(name: string): Promise<TodoHit[]> {
	return invoke<TodoHit[]>("scrape_todos", { name })
}

export async function contextPack(name: string): Promise<ContextPack> {
	return invoke<ContextPack>("context_pack", { name })
}

export async function openPathExternal(path: string): Promise<void> {
	return invoke<void>("open_path_external", { path })
}

export async function listSavedFilters(): Promise<SavedFilterDto[]> {
	return invoke<SavedFilterDto[]>("list_saved_filters")
}

export async function saveSavedFilter(
	id: string,
	label: string,
	query: string,
	createdAt?: string,
): Promise<SavedFilterDto> {
	return invoke<SavedFilterDto>("save_saved_filter", {
		id,
		label,
		query,
		createdAt: createdAt ?? null,
	})
}

export async function removeSavedFilter(id: string): Promise<void> {
	return invoke<void>("remove_saved_filter", { id })
}

export async function listSnapshots(): Promise<SnapshotInfo[]> {
	return invoke<SnapshotInfo[]>("list_snapshots")
}

export async function restoreSnapshot(path: string): Promise<void> {
	return invoke<void>("restore_snapshot", { path })
}

export async function deleteSnapshot(path: string): Promise<void> {
	return invoke<void>("delete_snapshot", { path })
}

export async function openUrlExternal(url: string): Promise<void> {
	return invoke<void>("open_path_external", { path: url })
}

// Copy text to the OS clipboard. Uses the Tauri-injected `__TAURI__` writeText
// if available, otherwise falls back to the browser API (works in WebView2).
export async function copyText(text: string): Promise<void> {
	try {
		const w = window as unknown as {
			__TAURI__?: {
				clipboard?: { writeText: (s: string) => Promise<void> }
			}
		}
		if (w.__TAURI__?.clipboard?.writeText) {
			await w.__TAURI__.clipboard.writeText(text)
			return
		}
	} catch {
		// ignore
	}
	await navigator.clipboard.writeText(text)
}

// Re-export the shell opener for places that want to do their own thing.
export { openExternal }
