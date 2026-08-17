import * as Sentry from "@sentry/svelte"
import App from "./App.svelte"
import "./app.css"

const sentryDsn = import.meta.env.VITE_SENTRY_DSN as string | undefined
if (sentryDsn) {
	Sentry.init({
		dsn: sentryDsn,
		release: `vobes-desktop@${__APP_VERSION__}`,
		environment: import.meta.env.DEV ? "development" : "production",
		tracesSampleRate: 0,
	})
}

function showFatal(msg: string) {
	let el = document.getElementById("fatal")
	if (!el) {
		el = document.createElement("div")
		el.id = "fatal"
		document.body.appendChild(el)
	}
	el.textContent = `Vobes error: ${msg}`
	el.style.cssText =
		"position:fixed;left:0;right:0;bottom:0;z-index:9999;background:#ef4444;color:#fff;padding:10px 14px;font:13px monospace;white-space:pre-wrap;"
}

window.addEventListener("error", (e) => showFatal(e.message))
window.addEventListener("unhandledrejection", (e) =>
	showFatal(String((e as PromiseRejectionEvent).reason)),
)

const app = new App({
	target: document.getElementById("app")!,
})

export default app