//! `vbs scan` — discover projects in configured roots.
//!
//! Same behaviour as the desktop's scan: a project already tracked is
//! refreshed (its git state, language and framework), not skipped. The
//! old version skipped known paths, so a scan never picked up a project
//! whose stack had changed.

use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::commands::shared::{absolute_normalized, vobe_from_detection};
use vobes_cli::app::App;

pub fn run(app: &App) -> Result<()> {
    let roots = app.config.resolved_roots();
    if roots.is_empty() {
        eprintln!(
            "no scan roots configured. edit {} or run `vbs init`.",
            app.config_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        );
        return Ok(());
    }

    let mut added = 0usize;
    let mut updated = 0usize;
    let bar = indicatif::ProgressBar::new(roots.len() as u64);

    for root in &roots {
        if !root.exists() {
            bar.println(format!("skip missing root: {}", root.display()));
            bar.inc(1);
            continue;
        }
        bar.println(format!("scanning {}", root.display()));
        let pairs = app.scanner.scan(root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            let existing = app.store.get_vobe_by_path(&path)?;
            let mut vobe = vobe_from_detection(&path, &detection)?;
            if let Some(prev) = existing {
                // Preserve identity + user fields, refresh the rest.
                vobe.id = prev.id;
                vobe.created_at = prev.created_at;
                vobe.name = prev.name;
                vobe.tags = prev.tags;
                vobe.notes = prev.notes;
                vobe.pinned = prev.pinned;
                vobe.last_opened = prev.last_opened;
                vobe.touch_modified();
                app.store.upsert_vobe(&vobe)?;
                updated += 1;
            } else {
                app.store.upsert_vobe(&vobe)?;
                app.store.record_activity(
                    &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Scanned)
                        .with_detail("vbs scan"),
                )?;
                vobe.touch_modified();
                added += 1;
                bar.println(format!("  + {} ({})", vobe.name, vobe.path.display()));
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    println!(
        "scanned {added} new, {updated} refreshed across {} roots",
        roots.len()
    );
    Ok(())
}
