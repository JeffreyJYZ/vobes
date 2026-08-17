// Auto-update helper: check for a newer release on startup and surface
// it as a toast. The full download + install + relaunch flow is
// triggered from a toast action so the user always opts in.

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { pushToast } from "./stores";

export async function checkForUpdate(): Promise<void> {
	let update: Awaited<ReturnType<typeof check>>;
	try {
		update = await check();
	} catch (e) {
		const msg = e && typeof e === "object" && "message" in e
			? String((e as { message: unknown }).message)
			: String(e);
		eprintln(`vobes: update check failed: ${msg}`);
		return;
	}
	if (!update) return;

	const ver = update.version;
	pushToast({
		kind: "info",
		message: `Update available: v${ver}`,
		action: {
			label: "Install & restart",
			run: async () => {
				await update.downloadAndInstall();
				await relaunch();
			},
		},
	});
}

function eprintln(msg: string): void {
	console.warn(msg);
}