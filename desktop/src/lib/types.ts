// Vobe model + nested types — mirror the Rust DTOs in `desktop/src-tauri/src/dto.rs`.

export type Commit = {
	hash: string
	message: string
	author: string
	date: string
}

export type GitInfo = {
	branch: string
	dirty: boolean
	ahead: number
	behind: number
	last_commit?: Commit | null
}

export type Vobe = {
	id: string
	name: string
	path: string
	framework: string | null
	language: string | null
	package_manager: string | null
	created_at: string
	last_opened: string | null
	last_modified: string | null
	tags: string[]
	notes: string | null
	pinned: boolean
	git: GitInfo | null
}

export type ActivityEvent = {
	id: number | null
	vobe_id: string
	kind: string
	timestamp: string
	detail: string | null
	actor: string
}

export type ScanConfig = {
	roots: string[]
	exclude: string[]
	max_depth: number
	follow_symlinks: boolean
}

export type DisplayConfig = {
	theme: "auto" | "light" | "dark"
}

export type Config = {
	scan: ScanConfig
	display: DisplayConfig
	desktop: DesktopConfig
}

export type DesktopConfig = {
	notify_behind: boolean
}

export type Paths = {
	config: string
	db: string
	snapshots: string
	state_dir: string
}

export type ConfigDto = {
	path: string
	paths: Paths
	config: Config
}

export type ViewId = "dashboard" | "projects" | "activity" | "settings"

export type SortKey = "name" | "last_modified" | "last_opened" | "created_at"

export type ToastKind = "info" | "success" | "error"

export type Toast = {
	id: number
	kind: ToastKind
	message: string
	ttl: number
	action?: {
		label: string
		run: () => void | Promise<void>
	}
}

export type TodoHit = {
	kind: "TODO" | "FIXME" | "XXX"
	line: number
	file: string
	text: string
}

export type ContextPack = {
	vobe: Vobe
	activity: ActivityEvent[]
	directory: string[]
	generated_at: string
}

export type SavedFilter = {
	id: string
	label: string
	query: string
	createdAt: string
}

// Backend-side DTO mirrors `SavedFilterDto` so the frontend can
// round-trip the same shape it sends to the store.
export type SavedFilterDto = {
	id: string
	label: string
	query: string
	created_at: string
}

export type SnapshotInfo = {
	path: string
	name: string
	size_bytes: number
	modified_at: string
}

export type TerminalApp = {
	id: string
	label: string
	is_default: boolean
}

export type EditorApp = {
	id: string
	label: string
	is_default: boolean
}
