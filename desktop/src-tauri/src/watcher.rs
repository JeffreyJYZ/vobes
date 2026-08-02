//! File-system watcher: debounced notifications of changes inside any
//! tracked vobe's path. When something changes we emit a Tauri event
//! the frontend can listen on to refresh in place.
//!
//! We intentionally use `notify-debouncer-mini` to coalesce the storm
//! of events that file editors produce during a save. The frontend
//! debounces again on its side for the same reason.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use tauri::{AppHandle, Emitter};
use vobes_store::Store;

use crate::ctx::DesktopCtx;

/// Active debouncer, kept alive for the lifetime of the app.
pub struct VobesWatcher {
    /// Holding the debouncer keeps its background thread running.
    /// We never call it; the field exists so it isn't dropped.
    _debouncer: Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,
}

impl VobesWatcher {
    /// Start a watcher over every vobe's path. Failures are logged
    /// but non-fatal — the app should still work without watching.
    pub fn start(ctx: Arc<DesktopCtx>, app: AppHandle) -> Self {
        let paths: Vec<PathBuf> = match ctx
            .store
            .list_vobes(&vobes_store::Filter::all(), vobes_store::Sort::Name)
        {
            Ok(vs) => vs.into_iter().map(|v| v.path).collect(),
            Err(e) => {
                eprintln!("vobes: watcher could not list vobes: {e}");
                Vec::new()
            }
        };

        let app_for_events = app.clone();
        let debouncer = match new_debouncer(
            Duration::from_millis(400),
            move |res: notify_debouncer_mini::DebounceEventResult| match res {
                Ok(events) => {
                    if !events.is_empty() {
                        app_for_events.emit("vobes://fs-changed", events.len()).ok();
                    }
                }
                Err(e) => eprintln!("vobes: watcher error: {e:?}"),
            },
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("vobes: watcher init failed: {e}");
                return Self { _debouncer: None };
            }
        };

        let mut debouncer = debouncer;
        for p in &paths {
            if p.exists() {
                if let Err(e) = debouncer.watcher().watch(p, RecursiveMode::Recursive) {
                    eprintln!("vobes: watcher cannot watch {}: {e}", p.display());
                }
            }
        }

        Self {
            _debouncer: Some(debouncer),
        }
    }
}
