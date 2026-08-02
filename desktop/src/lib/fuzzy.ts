// Small, dependency-free fuzzy scorer.
//
// Scoring rules (higher = better match):
//   - exact substring (case-insensitive):  huge bonus
//   - prefix match:                         big bonus
//   - consecutive character matches:        bonus
//   - matches at word boundaries:           bonus
//   - unmatched characters:                 small penalty
//   - shorter haystacks rank higher than longer ones with same hit set
//
// Returns 0 when no characters match, in which case the candidate is
// filtered out. Always pass the query lower-cased; we do the same for
// the haystack inside.

export interface FuzzyCandidate {
	/** Stable id used to re-find the source object. */
	id: string
	/** Primary text to match against. */
	text: string
	/** Optional secondary text (path, tags) — included in the haystack. */
	meta?: string
	/** Optional precomputed object to return alongside the score. */
	data?: unknown
}

export interface FuzzyResult<T = unknown> {
	id: string
	text: string
	meta?: string
	data?: T
	score: number
	/** Indices in `text` (0-based) that matched, for highlighting. */
	matches: number[]
}

export function fuzzy<T = unknown>(
	query: string,
	candidates: FuzzyCandidate[],
	limit = 50,
): FuzzyResult<T>[] {
	const q = query.trim().toLowerCase()
	if (q.length === 0) {
		return candidates.slice(0, limit).map((c) => ({
			id: c.id,
			text: c.text,
			meta: c.meta,
			data: c.data as T,
			score: 0,
			matches: [],
		}))
	}
	const out: FuzzyResult<T>[] = []
	for (const cand of candidates) {
		const hay = cand.meta ? `${cand.text} ${cand.meta}` : cand.text
		const score = scoreOne(q, hay.toLowerCase())
		if (score <= 0) continue
		out.push({
			id: cand.id,
			text: cand.text,
			meta: cand.meta,
			data: cand.data as T,
			score,
			matches: findMatchIndices(q, cand.text.toLowerCase()),
		})
	}
	out.sort((a, b) => b.score - a.score)
	return out.slice(0, limit)
}

function scoreOne(q: string, hay: string): number {
	if (!hay) return 0
	// Exact substring match is a strong signal.
	if (hay.includes(q)) {
		const startBonus = hay.startsWith(q) ? 50 : 0
		return 200 + startBonus - Math.min(hay.length, 50)
	}
	// Otherwise, walk the query through the haystack in order.
	let qi = 0
	let hi = 0
	let score = 0
	let lastMatch = -2
	let prevWasMatch = false
	let matches = 0
	while (qi < q.length && hi < hay.length) {
		if (q[qi] === hay[hi]) {
			matches++
			// Consecutive bonus.
			if (hi === lastMatch + 1) score += 12
			// Word boundary bonus.
			const before = hi === 0 ? " " : hay[hi - 1]
			if (
				before === " " ||
				before === "/" ||
				before === "-" ||
				before === "_" ||
				before === "."
			) {
				score += 8
			}
			// First-letter-of-string bonus.
			if (hi === 0) score += 6
			// Camel-case boundary: previous is lowercase, current is uppercase.
			if (hi > 0 && /[a-z]/.test(hay[hi - 1]) && /[A-Z]/.test(hay[hi])) {
				score += 4
			}
			lastMatch = hi
			prevWasMatch = true
			qi++
		} else {
			// Tiny penalty for skipped characters, but only on transitions.
			if (prevWasMatch) score -= 1
			prevWasMatch = false
		}
		hi++
	}
	if (qi < q.length) return 0 // didn't consume whole query
	// All chars matched — base score scales with match density and penalizes length.
	const density = matches / Math.max(hay.length, 1)
	return score + Math.floor(density * 30) - Math.min(hay.length, 20)
}

function findMatchIndices(q: string, text: string): number[] {
	const out: number[] = []
	let qi = 0
	for (let i = 0; i < text.length && qi < q.length; i++) {
		if (text[i] === q[qi]) {
			out.push(i)
			qi++
		}
	}
	return out
}
