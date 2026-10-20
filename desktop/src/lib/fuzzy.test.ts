import { describe, expect, it } from "vitest"
import { fuzzy } from "./fuzzy"

const mk = (text: string, meta?: string, id = text) => ({
	id,
	text,
	meta,
})

describe("fuzzy", () => {
	it("returns all candidates (unsorted, up to limit) on an empty query", () => {
		const out = fuzzy("", [mk("alpha"), mk("beta"), mk("gamma")])
		expect(out).toHaveLength(3)
		expect(out.map((r) => r.text)).toEqual(["alpha", "beta", "gamma"])
		expect(out.every((r) => r.score === 0 && r.matches.length === 0)).toBe(
			true,
		)
	})

	it("respects the limit on an empty query", () => {
		const out = fuzzy("", [mk("a"), mk("b"), mk("c"), mk("d")], 2)
		expect(out).toHaveLength(2)
	})

	it("filters out candidates that do not contain all query chars in order", () => {
		const out = fuzzy("abc", [mk("axbxc"), mk("xyz")])
		expect(out.map((r) => r.text)).toEqual(["axbxc"])
	})

	it("ranks an exact substring match above a scattered one", () => {
		const out = fuzzy("foo", [mk("bar foo baz"), mk("f-o-o")])
		expect(out[0].text).toBe("bar foo baz")
	})

	it("gives a prefix match a higher score than a non-prefix substring", () => {
		const out = fuzzy("foo", [mk("a foo"), mk("foobar")])
		expect(out[0].text).toBe("foobar")
	})

	it("matches against meta (secondary text) too", () => {
		const out = fuzzy("rust", [mk("engine", "~/code/rust/engine.rs")])
		expect(out).toHaveLength(1)
		expect(out[0].id).toBe("engine")
	})

	it("reports match indices within text (lower-cased) for highlighting", () => {
		const out = fuzzy("ob", [mk("foobar")])
		expect(out[0].matches).toEqual([1, 3])
	})

	it("is case-insensitive", () => {
		const out = fuzzy("FOO", [mk("FooBar")])
		expect(out).toHaveLength(1)
	})
})
