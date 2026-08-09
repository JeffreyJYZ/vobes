// Svelte stores — the in-memory picture of the world.
//
// Everything that mutates app state lives here so individual components
// stay declarative. Stores are plain `writable`s, no SvelteKit magic —
// the app is a single window.

import { derived, get, type Readable, writable } from "svelte/store"
import * as api from "./api"
import type {
	ActivityEvent,
	Config,
	ConfigDto,
	Paths,
	SavedFilter,
	SavedFilterDto,
	SortKey,
	Toast,
	ToastKind,
	ViewId,
	Vobe,
} from "./types"

// ---- Vobes + activity ----

export const vobes = writable<Vobe[]>([])
export const activity = writable<ActivityEvent[]>([])
export const lastRefreshed = writable<number>(0)
export const scanning = writable<boolean>(false)

/** A short human label for activity `kind` strings coming from Rust. */
export function kindLabel(k: string): string {
	switch (k) {
		case "Opened":
			return "opened"
		case "Modified":
			return "modified"
		case "Committed":
			return "committed"
		case "Scanned":
			return "scanned"
		case "Created":
			return "created"
		case "Closed":
			return "closed"
		case "Tagged":
			return "tagged"
		case "Noted":
			return "noted"
		default:
			return k.toLowerCase()
	}
}

export async function refresh(opts: { silent?: boolean } = {}): Promise<void> {
	if (get(scanning)) return
	if (!opts.silent) scanning.set(true)
	try {
		const [vs, acts] = await Promise.all([
			api.listVobes(),
			api.recentActivity(200),
		])
		vobes.set(vs)
		activity.set(acts)
		lastRefreshed.set(Date.now())
		// If the user has any tracked vobes, onboarding is effectively done.
		if (vs.length > 0 && !get(onboardingDone)) {
			if (typeof localStorage !== "undefined") {
				localStorage.setItem("vobes:onboarded", "1")
			}
			onboardingDone.set(true)
		}
		// Fire opt-in notifications, if any. The opt-in lives in
		// `config.desktop.notify_behind`; the throttle timestamp is
		// local runtime state, not config.
		const cfg = get(config)
		if (cfg?.desktop?.notify_behind) {
			notifyBehindSummary(vs, acts.length)
		}
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	} finally {
		scanning.set(false)
	}
}

async function notifyBehindSummary(allVobes: Vobe[], _recentEvents: number) {
	try {
		const behind = allVobes.filter((v) => v.git && v.git.behind > 0)
		if (behind.length === 0) return
		// Throttle: don't fire more than one notification per 5 min.
		const last = Number(
			localStorage.getItem("vobes:notify-behind-at") ?? "0",
		)
		if (Date.now() - last < 5 * 60_000) return
		localStorage.setItem("vobes:notify-behind-at", String(Date.now()))
		const mod = await import("@tauri-apps/plugin-notification")
		const granted = await mod.isPermissionGranted()
		if (!granted) {
			const ok = await mod.requestPermission()
			if (ok !== "granted") return
		}
		const top = behind
			.sort((a, b) => (b.git?.behind ?? 0) - (a.git?.behind ?? 0))
			.slice(0, 3)
			.map((v) => `${v.name} (↓${v.git?.behind})`)
			.join(", ")
		mod.sendNotification({
			title: `${behind.length} vobe${behind.length === 1 ? "" : "s"} behind upstream`,
			body: top + (behind.length > 3 ? "…" : ""),
		})
	} catch {
		// Plugin missing or OS denies — silent.
	}
}

export async function doScan(): Promise<void> {
	scanning.set(true)
	try {
		const found = await api.scan()
		pushToast({
			kind: "success",
			message:
				found > 0
					? `Found ${found} new vobe${found === 1 ? "" : "s"}.`
					: "No new vobes.",
		})
		await refresh({ silent: true })
		lastRefreshed.set(Date.now())
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	} finally {
		scanning.set(false)
	}
}

export async function doSync(): Promise<void> {
	scanning.set(true)
	try {
		const [added, updated] = await api.sync()
		pushToast({
			kind: "success",
			message: `Sync: +${added} new, ${updated} refreshed.`,
		})
		await refresh({ silent: true })
		lastRefreshed.set(Date.now())
	} catch (e) {
		pushToast({ kind: "error", message: errorString(e) })
	} finally {
		scanning.set(false)
	}
}

// ---- View + UI state ----

export const view = writable<ViewId>("dashboard")
export const selectedVobe = writable<Vobe | null>(null)
export const searchQuery = writable<string>("")
export const sortKey = writable<SortKey>("last_modified")
export const density = writable<"comfy" | "compact">("comfy")

// ---- Command palette ----

export type PaletteMode = "closed" | "default"
export const palette = writable<{ mode: PaletteMode }>({ mode: "closed" })

export function openPalette() {
	palette.set({ mode: "default" })
}
export function closePalette() {
	palette.set({ mode: "closed" })
}
export function togglePalette() {
	palette.update((p) => ({
		mode: p.mode === "closed" ? "default" : "closed",
	}))
}

// ---- Shortcut help ----
export const helpOpen = writable<boolean>(false)
export function toggleHelp() {
	helpOpen.update((v) => !v)
}

// ---- Onboarding ----
export const onboardingDone = writable<boolean>(
	typeof localStorage !== "undefined" &&
		localStorage.getItem("vobes:onboarded") === "1",
)

// ---- Settings / config ----

export const configPath = writable<string>("")
export const config = writable<Config | null>(null)
export const configPaths = writable<Paths>({
	config: "",
	db: "",
	snapshots: "",
	state_dir: "",
})

export async function loadConfig(): Promise<void> {
	try {
		const dto: ConfigDto = await api.getConfig()
		config.set(dto.config)
		configPath.set(dto.path)
		configPaths.set(dto.paths)
	} catch (e) {
		pushToast({
			kind: "error",
			message: `Failed to load config: ${errorString(e)}`,
		})
	}
}

export async function applyConfig(next: Config): Promise<void> {
	try {
		const dto = await api.saveConfig(next)
		config.set(dto.config)
		configPath.set(dto.path)
		configPaths.set(dto.paths)
		// User has engaged with settings — onboarding is over.
		if (typeof localStorage !== "undefined") {
			localStorage.setItem("vobes:onboarded", "1")
		}
		onboardingDone.set(true)
	} catch (e) {
		pushToast({
			kind: "error",
			message: `Failed to save config: ${errorString(e)}`,
		})
		throw e
	}
}

// ---- Toasts ----

export const toasts = writable<Toast[]>([])
let toastSeq = 1

export function pushToast(input: {
	kind?: ToastKind
	message: string
	ttl?: number
}): number {
	const id = toastSeq++
	const t: Toast = {
		id,
		kind: input.kind ?? "info",
		message: input.message,
		ttl: input.ttl ?? 3500,
	}
	toasts.update((arr) => [...arr, t])
	if (t.ttl > 0) {
		setTimeout(() => dismissToast(id), t.ttl)
	}
	return id
}

export function dismissToast(id: number) {
	toasts.update((arr) => arr.filter((t) => t.id !== id))
}

// ---- Derived helpers ----

export const attentionCount: Readable<number> = derived(vobes, ($v) => {
	let n = 0
	for (const v of $v) {
		if (v.git) {
			if (v.git.dirty) n++
			else if (v.git.ahead > 0) n++
			else if (v.git.behind > 0) n++
		}
	}
	return n
})

export const visibleVobes: Readable<Vobe[]> = derived(
	[vobes, searchQuery, sortKey],
	([$vobes, $q, $sort]) => {
		const ql = $q.trim().toLowerCase()
		let arr = $vobes
		if (ql) {
			arr = arr.filter(
				(v) =>
					v.name.toLowerCase().includes(ql) ||
					(v.path ?? "").toLowerCase().includes(ql) ||
					v.tags.some((t) => t.toLowerCase().includes(ql)),
			)
		}
		return [...arr].sort((a, b) => sortVobe(a, b, $sort))
	},
)

export function sortVobe(a: Vobe, b: Vobe, key: SortKey): number {
	if (a.pinned !== b.pinned) return a.pinned ? -1 : 1
	switch (key) {
		case "name":
			return a.name.localeCompare(b.name)
		case "created_at":
			return (b.created_at ?? "").localeCompare(a.created_at ?? "")
		case "last_opened":
			return (b.last_opened ?? "").localeCompare(a.last_opened ?? "")
		default:
			return (b.last_modified ?? "").localeCompare(a.last_modified ?? "")
	}
}

export function errorString(e: unknown): string {
	if (!e) return "Unknown error"
	if (typeof e === "string") return e
	if (typeof e === "object") {
		const obj = e as Record<string, unknown>
		if (typeof obj.message === "string") return obj.message
		if (typeof obj.msg === "string") return obj.msg
		try {
			return JSON.stringify(e)
		} catch {
			return String(e)
		}
	}
	return String(e)
}

// ---- Theme ----
//
// `theme` reflects the user's preference from config (`auto`/`light`/`dark`).
// `effective` is what the OS actually sees after resolving `auto`.

export type ThemePref = "auto" | "light" | "dark"
export type EffectiveTheme = "light" | "dark"

export const themePref = writable<ThemePref>("auto")
export const effectiveTheme = writable<EffectiveTheme>(
	typeof window !== "undefined" &&
		window.matchMedia?.("(prefers-color-scheme: dark)").matches
		? "dark"
		: "light",
)

if (typeof window !== "undefined" && window.matchMedia) {
	const mq = window.matchMedia("(prefers-color-scheme: dark)")
	const listener = (e: MediaQueryListEvent) => {
		effectiveTheme.set(e.matches ? "dark" : "light")
	}
	if (mq.addEventListener) mq.addEventListener("change", listener)
	else mq.addListener(listener)
}

// Apply the current effective theme to the document. Components don't
// need to worry about it — CSS hooks off [data-theme="dark"].
export function applyTheme() {
	if (typeof document === "undefined") return
	const t = get(effectiveTheme)
	document.documentElement.setAttribute("data-theme", t)
}

effectiveTheme.subscribe(() => applyTheme())

// ---- Saved filters ----
//
// Backed by SQLite via the `saved_filters` table (schema v4) so
// views persist across machines when the user exports their
// snapshot. On first run after the upgrade, any filters previously
// stored in localStorage are migrated to the backend and the
// local copy is cleared.

export const savedFilters = writable<SavedFilter[]>([])
export const savedFiltersLoaded = writable(false)

function dtoToSavedFilter(d: SavedFilterDto): SavedFilter {
	return {
		id: d.id,
		label: d.label,
		query: d.query,
		createdAt: d.created_at,
	}
}

function savedFilterToDto(f: SavedFilter): SavedFilterDto {
	return {
		id: f.id,
		label: f.label,
		query: f.query,
		created_at: f.createdAt,
	}
}

// Load saved filters from the backend, and migrate any localStorage
// carryover from pre-v0.1.4 builds in the same pass.
export async function loadSavedFilters(): Promise<void> {
	if (get(savedFiltersLoaded)) return
	try {
		// One-time migration: push any localStorage filters into the
		// backend, then drop the local copy.
		let migrated: SavedFilter[] = []
		try {
			const raw = localStorage.getItem("vobes:saved-filters")
			if (raw) {
				const arr = JSON.parse(raw) as SavedFilter[]
				if (Array.isArray(arr) && arr.length > 0) migrated = arr
			}
		} catch {
			// ignore
		}
		if (migrated.length > 0) {
			for (const f of migrated) {
				await api.saveSavedFilter(f.id, f.label, f.query, f.createdAt)
			}
			try {
				localStorage.removeItem("vobes:saved-filters")
			} catch {
				// ignore
			}
		}
		const dtos = await api.listSavedFilters()
		savedFilters.set(dtos.map(dtoToSavedFilter))
		savedFiltersLoaded.set(true)
	} catch (e) {
		// Backend unavailable (e.g. dev-mode race) — leave the store
		// empty; refresh will retry.
		console.warn("loadSavedFilters failed:", e)
	}
}

export async function addSavedFilter(
	label: string,
	query: string,
): Promise<SavedFilter> {
	const f: SavedFilter = {
		id: `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
		label: label.trim() || query.trim(),
		query: query.trim(),
		createdAt: new Date().toISOString(),
	}
	await api.saveSavedFilter(
		f.id,
		f.label,
		f.query,
		f.createdAt,
	)
	savedFilters.update((arr) => [f, ...arr].slice(0, 50))
	return f
}

export async function removeSavedFilter(id: string): Promise<void> {
	try {
		await api.removeSavedFilter(id)
	} catch (e) {
		console.warn("removeSavedFilter backend call failed:", e)
	}
	savedFilters.update((arr) => arr.filter((f) => f.id !== id))
}

// Convenience sync writer for callers that already have the dto in
// hand and do not need to await the IPC round-trip.
export function setSavedFilters(arr: SavedFilter[]): void {
	savedFilters.set(arr)
}
