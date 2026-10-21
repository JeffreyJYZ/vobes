//! `vbs sync` — re-scan roots, refresh git cache, record activity.

use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::commands::shared::{absolute_normalized, vobe_from_detection};
use vobes_cli::app::App;

pub fn run(app: &App) -> Result<()> {
    let roots = app.config.resolved_roots();
    let mut new = 0usize;
    let mut changed = 0usize;
    let mut unchanged = 0usize;

    for root in &roots {
        if !root.exists() {
            continue;
        }
        let pairs = app.scanner.scan(root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            let existing = app.store.get_vobe_by_path(&path)?;
            let detected = vobe_from_detection(&path, &detection)?;
            if let Some(prev) = existing {
                // Preserve identity + user fields, refresh the rest.
                let (merged, did_change) = prev.merge_detected(&detected);
                app.store.upsert_vobe(&merged)?;
                if did_change {
                    changed += 1;
                } else {
                    unchanged += 1;
                }
            } else {
                app.store.upsert_vobe(&detected)?;
                app.store.record_activity(
                    &ActivityEvent::now_env(detected.id.clone(), ActivityKind::Scanned)
                        .with_detail("vbs sync"),
                )?;
                new += 1;
            }
        }
    }

    println!("synced: {new} new, {changed} changed, {unchanged} unchanged");
    Ok(())
}
