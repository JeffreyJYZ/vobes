//! Vobes desktop backend. Wires Tauri commands to the shared core.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

mod commands;
mod ctx;
mod dto;

pub use ctx::DesktopCtx;

use tauri::Manager;

/// Entry point invoked by `main.rs`.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let ctx = match DesktopCtx::load() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("vobes: failed to load app context: {e}");
                    return Err(e.into());
                }
            };
            app.manage(std::sync::Arc::new(ctx));
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
