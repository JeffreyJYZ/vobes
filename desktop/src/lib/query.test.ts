import { describe, expect, it } from "bun:test"
import { parseQuery, vobeMatches } from "./stores"
import type { Vobe } from "./types"

const mk = (over: Partial<Vobe> = {}): Vobe =>
	({
		id: "v1",
		name: "my-app",
		path: "/code/alpha/beta",
		tags: ["backend"],
		language: "Rust",
		framework: "Tauri",
		package_manager: "cargo",
		pinned: false,
		created_at: "2026-01-01T00:00:00Z",
		...over,
	}) as Vobe

describe("parseQuery", () => {
	it("keeps each free-text term separate", () => {
		const p = parseQuery("alpha beta")
		expect(p.terms).toEqual(["alpha", "beta"])
	})

	it("matches every term independently", () => {
		const p = parseQuery("alpha beta")
		expect(vobeMatches(mk(), p)).toBe(true)
		expect(vobeMatches(mk({ path: "/code/alpha" }), p)).toBe(false)
	})

	it("supports name:", () => {
		const p = parseQuery("name:my-app")
		expect(p.terms).toEqual([])
		expect(vobeMatches(mk(), p)).toBe(true)
		expect(vobeMatches(mk({ name: "other" }), p)).toBe(false)
	})

	it("supports is:attention", () => {
		const p = parseQuery("is:attention")
		expect(p.predicates).toHaveLength(1)
		expect(vobeMatches(mk({ git: { dirty: true } } as never), p)).toBe(true)
		expect(vobeMatches(mk({ git: { behind: 2 } } as never), p)).toBe(true)
		expect(vobeMatches(mk({ git: { dirty: false } } as never), p)).toBe(
			false,
		)
		expect(vobeMatches(mk(), p)).toBe(false)
	})

	it("still honours tag: alongside free text", () => {
		const p = parseQuery("tag:backend alpha")
		expect(p.tags).toEqual(["backend"])
		expect(vobeMatches(mk(), p)).toBe(true)
		expect(vobeMatches(mk({ tags: ["other"] }), p)).toBe(false)
	})
})
