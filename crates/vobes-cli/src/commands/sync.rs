//! `vbs sync` — re-scan roots, refresh git cache, record activity.

use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::commands::shared::{absolute_normalized, vobe_from_detection};
use vobes_cli::app::App;

pub fn run(app: &App) -> Result<()> {
    let roots = app.config.resolved_roots();
    let mut updated = 0usize;
    let mut added = 0usize;

    for root in &roots {
        if !root.exists() {
            continue;
        }
        let pairs = app.scanner.scan(root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            let existing = app.store.get_vobe_by_path(&path)?;
            let mut vobe = vobe_from_detection(&path, &detection)?;
            if let Some(prev) = existing {
                // Preserve identity + user fields from the existing vobe.
                vobe.id = prev.id;
                vobe.created_at = prev.created_at;
                vobe.tags = prev.tags;
                vobe.notes = prev.notes;
                vobe.metadata = prev.metadata;
                vobe.last_opened = prev.last_opened;
                vobe.touch_modified();
                app.store.upsert_vobe(&vobe)?;
                updated += 1;
            } else {
                app.store.upsert_vobe(&vobe)?;
                app.store.record_activity(
                    &ActivityEvent::now(vobe.id.clone(), ActivityKind::Scanned)
                        .with_detail("vbs sync"),
                )?;
                added += 1;
            }
        }
    }

    println!("synced: {added} added, {updated} updated");
    Ok(())
}
