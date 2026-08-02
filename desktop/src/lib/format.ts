// Tiny formatting helpers — date relative-to-now, short path,
// elapsed timers. All keep deps at zero.

export function relativeTime(ts: string | null | undefined): string {
	if (!ts) return "—"
	const d = new Date(ts)
	const t = d.getTime()
	if (Number.isNaN(t)) return "—"
	const s = (Date.now() - t) / 1000
	if (s < 5) return "just now"
	if (s < 60) return `${Math.floor(s)}s ago`
	if (s < 3600) return `${Math.floor(s / 60)}m ago`
	if (s < 86400) return `${Math.floor(s / 3600)}h ago`
	if (s < 604800) return `${Math.floor(s / 86400)}d ago`
	if (s < 2592000) return `${Math.floor(s / 604800)}w ago`
	if (s < 31536000) return `${Math.floor(s / 2592000)}mo ago`
	return `${Math.floor(s / 31536000)}y ago`
}

export function absoluteTime(ts: string | null | undefined): string {
	if (!ts) return "—"
	const d = new Date(ts)
	if (Number.isNaN(d.getTime())) return "—"
	return d.toLocaleString()
}

export function shortPath(p: string | null | undefined, max = 60): string {
	if (!p) return "—"
	if (p.length <= max) return p
	const head = p.slice(0, 12)
	const tail = p.slice(-(max - head.length - 3))
	return `${head}…${tail}`
}

export function basename(p: string | null | undefined): string {
	if (!p) return "—"
	const m = p.replace(/\/+$/, "").split(/[/\\]/)
	return m[m.length - 1] || p
}

// Friendly defaults for "no detection". Plain text is friendlier than
// a dash and answers the implicit question "what kind of project is this?"
// — "it's a vanilla one, nothing fancy going on."
export function languageLabel(lang: string | null | undefined): string {
	if (lang?.trim()) return lang
	return "Vanilla"
}

export function frameworkLabel(fw: string | null | undefined): string {
	if (fw?.trim()) return fw
	return "Vanilla"
}

export function packageManagerLabel(pm: string | null | undefined): string {
	if (pm?.trim()) return pm
	return "None"
}

export function parentPath(p: string | null | undefined): string {
	if (!p) return "—"
	const trimmed = p.replace(/\/+$/, "")
	const idx = Math.max(trimmed.lastIndexOf("/"), trimmed.lastIndexOf("\\"))
	if (idx < 0) return trimmed
	if (idx === 0) return trimmed.slice(0, 1)
	return trimmed.slice(0, idx)
}

export function isMac(): boolean {
	if (typeof navigator === "undefined") return false
	return /Mac/i.test(navigator.platform)
}

export function modKeyLabel(): string {
	return isMac() ? "⌘" : "Ctrl"
}

export function enterKeyLabel(): string {
	return isMac() ? "↵" : "Enter"
}
