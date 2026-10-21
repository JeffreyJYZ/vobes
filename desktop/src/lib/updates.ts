// Auto-update surface. One state machine shared by the in-app notice and
// the Settings section, so a check started in either place is visible in
// both and neither owns the flow.
//
// Two paths are tried: the Tauri updater (verifies the signature and knows
// this platform's artifact), then the GitHub releases API (a version to
// link to when the plugin is absent or has nothing for this platform).

import { getVersion } from "@tauri-apps/api/app"
import { relaunch } from "@tauri-apps/plugin-process"
import {
	check,
	type DownloadEvent,
	type Update,
} from "@tauri-apps/plugin-updater"
import { get, writable } from "svelte/store"

const REPO = "JeffreyJYZ/vobes"
const LATEST_RELEASE_API = `https://api.github.com/repos/${REPO}/releases/latest`
const DISMISSED_KEY = "vobes:dismissed-update"

export const RELEASES_URL = `https://github.com/${REPO}/releases`
export const LATEST_RELEASE_URL = `${RELEASES_URL}/latest`

export type UpdatePhase =
	| "idle"
	| "checking"
	| "current"
	| "available"
	| "downloading"
	| "failed"

/// A newer release, tagged with what can be done about it: `updater` carries
/// the handle `installNow` needs, `github` is only a version to link to.
export type UpdateOffer =
	| { kind: "updater"; version: string; update: Update }
	| { kind: "github"; version: string }

export interface UpdateState {
	phase: UpdatePhase
	running: string | null
	offer: UpdateOffer | null
	percent: number | null
	message: string | null
	/// When the last check settled, for the "checked 5 minutes ago" line.
	checkedAt: number | null
	/// Why the Tauri updater path found nothing, kept apart from `message`
	/// so a working GitHub fallback does not hide a broken plugin.
	pluginError: string | null
}

const initial: UpdateState = {
	phase: "idle",
	running: null,
	offer: null,
	percent: null,
	message: null,
	checkedAt: null,
	pluginError: null,
}

export const updates = writable<UpdateState>(initial)

function patch(next: Partial<UpdateState>): void {
	updates.update((s) => ({ ...s, ...next }))
}

/// Dot-numbered comparison; anything that is not exactly `x.y.z` is not newer.
export function isNewer(latest: string, current: string): boolean {
	const pattern = /^\d+\.\d+\.\d+$/
	if (!pattern.test(latest) || !pattern.test(current)) return false
	const [lMaj, lMin, lPat] = latest.split(".").map(Number)
	const [cMaj, cMin, cPat] = current.split(".").map(Number)
	if (lMaj !== cMaj) return lMaj > cMaj
	if (lMin !== cMin) return lMin > cMin
	return lPat > cPat
}

/// Newest published release version, from the GitHub API.
async function latestVersion(): Promise<string> {
	const response = await fetch(LATEST_RELEASE_API, {
		headers: { accept: "application/vnd.github+json" },
	})
	if (!response.ok) {
		throw new Error(`release check failed (${response.status})`)
	}
	const payload = (await response.json()) as { tag_name?: unknown }
	const tag = payload.tag_name
	if (typeof tag !== "string" || tag === "") {
		throw new Error("release has no tag")
	}
	return tag.replace(/^v/, "")
}

/// Version the notice was dismissed for, persisted across launches.
export function dismissedVersion(): string | null {
	return typeof localStorage === "undefined"
		? null
		: localStorage.getItem(DISMISSED_KEY)
}

/// Back to `idle`, handing the Rust-side handle back first — a closed handle
/// cannot be installed from later, so the next check gets a fresh one.
export function discardOffer(): void {
	const { offer, running, checkedAt, pluginError } = get(updates)
	if (offer?.kind === "updater") {
		void offer.update.close().catch(() => {})
	}
	updates.set({ ...initial, running, checkedAt, pluginError })
}

export function dismissOffer(): void {
	const { offer } = get(updates)
	if (offer && typeof localStorage !== "undefined") {
		localStorage.setItem(DISMISSED_KEY, offer.version)
	}
	discardOffer()
}

/// Ask both paths for a newer release. A second call while one is in flight
/// is a no-op, so the notice and the Settings section can't race.
export async function checkForUpdates(): Promise<void> {
	const phase = get(updates).phase
	if (phase === "checking" || phase === "downloading") return
	const previous = get(updates).offer
	if (previous?.kind === "updater") {
		void previous.update.close().catch(() => {})
	}
	patch({
		phase: "checking",
		offer: null,
		percent: null,
		message: null,
		pluginError: null,
	})

	let running: string | null = null
	try {
		running = await getVersion()
	} catch {
		running = null
	}

	let offer: UpdateOffer | null = null
	let pluginError: string | null = null
	try {
		const found = await check()
		if (found) {
			offer = { kind: "updater", version: found.version, update: found }
		}
	} catch (e) {
		// Plugin absent, offline, or no artifact for this platform — the
		// GitHub path below still answers "is there a newer version", but
		// the reason stays visible in Settings.
		pluginError = e instanceof Error ? e.message : String(e)
	}

	let message: string | null = null
	if (!offer && running && !apiThrottled()) {
		try {
			const newest = await latestVersion()
			lastApiAt = Date.now()
			if (isNewer(newest, running)) {
				offer = { kind: "github", version: newest }
			}
		} catch (e) {
			lastApiAt = Date.now()
			const release = e instanceof Error ? e.message : String(e)
			message = pluginError ? `${pluginError}; ${release}` : release
		}
	}

	patch({
		phase: offer ? "available" : message ? "failed" : "current",
		running,
		offer,
		percent: null,
		message: offer ? null : message,
		checkedAt: Date.now(),
		pluginError,
	})
}

/// A periodic sweep that only runs when no offer is already waiting: an
/// installed app left open for days would otherwise never hear about a
/// release, which is what made "checks on launch" look like "never checks".
const POLL_MS = 30 * 1000
const FOCUS_STALE_MS = 10 * 1000

/// The Tauri path reads `releases/latest/download/latest.json`, a plain release
/// asset with no API budget. The GitHub API fallback is unauthenticated and
/// capped at 60 requests an hour per IP, so a 30 s poll would spend 120 of them:
/// it runs at most once every ten minutes and the plugin path keeps the fast
/// cadence.
const API_MIN_MS = 10 * 60 * 1000

let lastApiAt = 0

function apiThrottled(): boolean {
	return Date.now() - lastApiAt < API_MIN_MS
}

let watcher: (() => void) | null = null

/// Check now, then every 30 seconds, and on focus when the last check is
/// older than ten seconds. Idempotent — mounting twice keeps one timer.
export function startUpdateWatcher(): () => void {
	if (watcher) return watcher
	const sweep = (): void => {
		const { phase } = get(updates)
		if (
			phase === "checking" ||
			phase === "downloading" ||
			phase === "available"
		) {
			return
		}
		void checkForUpdates()
	}
	const onFocus = (): void => {
		const { checkedAt } = get(updates)
		if (checkedAt !== null && Date.now() - checkedAt < FOCUS_STALE_MS) {
			return
		}
		sweep()
	}
	void checkForUpdates()
	const timer = setInterval(sweep, POLL_MS)
	window.addEventListener("focus", onFocus)
	const stop = (): void => {
		clearInterval(timer)
		window.removeEventListener("focus", onFocus)
		watcher = null
	}
	watcher = stop
	return stop
}

/// Download and install the offered update, reporting 0-100 progress (`null`
/// while the total size is unknown), then relaunch.
///
/// `relaunch` is last because macOS and Linux only run the new version after
/// it; on Windows the installer exits the app first, so a settled call that
/// failed means the download or the restart did not complete.
export async function installNow(): Promise<void> {
	const { phase, offer } = get(updates)
	if (phase !== "available" || offer?.kind !== "updater") return
	const update = offer.update
	patch({ phase: "downloading", percent: null, message: null })

	let expected = 0
	let received = 0
	try {
		await update.downloadAndInstall((event: DownloadEvent) => {
			if (event.event === "Started") {
				expected = event.data.contentLength ?? 0
				received = 0
				patch({ percent: expected === 0 ? null : 0 })
				return
			}
			if (event.event !== "Progress") return
			received += event.data.chunkLength
			patch({
				percent:
					expected === 0
						? null
						: Math.min(
								100,
								Math.round((received / expected) * 100),
							),
			})
		})
		await relaunch()
	} catch (e) {
		patch({
			phase: "failed",
			percent: null,
			message: e instanceof Error ? e.message : String(e),
		})
	}
}
