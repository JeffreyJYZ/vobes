// Markdown rendering with sanitization. We never trust the input —
// README files can contain hostile HTML, and the content may also
// come from a deep link or another agent in the future.

import DOMPurify from "dompurify"
import { marked } from "marked"

marked.setOptions({
	gfm: true,
	breaks: false,
})

const ALLOWED_TAGS = [
	"a",
	"abbr",
	"b",
	"blockquote",
	"br",
	"code",
	"del",
	"div",
	"em",
	"h1",
	"h2",
	"h3",
	"h4",
	"h5",
	"h6",
	"hr",
	"i",
	"img",
	"ins",
	"kbd",
	"li",
	"mark",
	"ol",
	"p",
	"pre",
	"s",
	"span",
	"strong",
	"sub",
	"sup",
	"table",
	"tbody",
	"td",
	"th",
	"thead",
	"tr",
	"ul",
]

const ALLOWED_ATTR = [
	"href",
	"title",
	"alt",
	"src",
	"id",
	"class",
	"align",
	"colspan",
	"rowspan",
	"start",
]

DOMPurify.addHook("afterSanitizeAttributes", (node) => {
	// Force all links to open in the OS browser, never navigate the
	// Tauri webview itself.
	if (node.tagName === "A") {
		node.setAttribute("target", "_blank")
		node.setAttribute("rel", "noopener noreferrer")
	}
})

export function renderMarkdown(md: string): string {
	if (!md) return ""
	const raw = marked.parse(md, { async: false }) as string
	return DOMPurify.sanitize(raw, {
		ALLOWED_TAGS,
		ALLOWED_ATTR,
		FORBID_TAGS: ["script", "style", "iframe", "object", "embed"],
		FORBID_ATTR: ["onerror", "onload", "onclick", "onmouseover"],
	})
}
