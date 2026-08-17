//! Vobes desktop backend. Wires Tauri commands to the shared core.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

mod commands;
mod ctx;
mod dto;
mod platform;
mod watcher;

pub use ctx::DesktopCtx;

use std::sync::Arc;
use tauri::{Emitter, Manager};

use crate::watcher::VobesWatcher;

/// Entry point invoked by `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _sentry = init_sentry();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let ctx = Arc::new(DesktopCtx::load());
            app.manage(ctx.clone());

            // Register the deep-link URL scheme at runtime on Linux/Windows
            // (macOS picks it up from the bundle manifest).
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register("vobes") {
                    eprintln!("vobes: deep-link register failed: {e}");
                }
            }

            // Start the file watcher (optional — failures are non-fatal).
            let watcher = VobesWatcher::start(ctx.clone(), app.handle().clone());
            app.manage(watcher);

            // Register the global shortcut to summon the palette.
            // We pick Ctrl+Alt+V on all platforms — discoverable, low collision.
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };
                let app_handle = app.handle().clone();
                let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV);
                if let Err(e) = app.global_shortcut().register(shortcut) {
                    eprintln!("vobes: global shortcut register failed: {e}");
                } else {
                    let handle = app_handle.clone();
                    app.global_shortcut()
                        .on_shortcut(shortcut, move |_app, _sc, ev| {
                            if ev.state() == ShortcutState::Pressed {
                                handle.emit("vobes://show-palette", ()).ok();
                            }
                        })
                        .ok();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_vobes,
            commands::get_vobe,
            commands::recent_activity,
            commands::vobe_activity,
            commands::scan,
            commands::reset_and_rescan,
            commands::sync,
            commands::add_vobe,
            commands::remove_vobe,
            commands::open_vobe,
            commands::export_json,
            commands::get_config,
            commands::save_config,
            commands::open_in_terminal,
            commands::reveal_in_finder,
            commands::copy_to_clipboard,
            commands::save_notes,
            commands::set_pinned,
            commands::get_pinned,
            commands::set_tags,
            commands::read_readme,
            commands::scrape_todos,
            commands::context_pack,
            commands::open_path_external,
            commands::list_saved_filters,
            commands::save_saved_filter,
            commands::remove_saved_filter,
            commands::list_snapshots,
            commands::restore_snapshot,
            commands::delete_snapshot,
            commands::list_terminals,
            commands::list_editors,
            commands::open_in_terminal,
            commands::open_in_editor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize Sentry crash reporting. No-op when `VOBES_SENTRY_DSN`
/// is unset so dev builds stay local. The returned guard must be
/// kept alive for the process lifetime — dropping it at scope exit
/// would flush pending events too early.
fn init_sentry() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("VOBES_SENTRY_DSN")
        .ok()
        .filter(|s| !s.is_empty())?;
    let env = if cfg!(debug_assertions) {
        "development"
    } else {
        "production"
    };
    let opts = sentry::ClientOptions {
        dsn: Some(
            dsn.parse()
                .expect("VOBES_SENTRY_DSN is not a valid Sentry DSN"),
        ),
        release: Some(format!("vobes-desktop@{}", env!("CARGO_PKG_VERSION")).into()),
        environment: Some(env.into()),
        ..Default::default()
    };
    Some(sentry::init(opts))
}
