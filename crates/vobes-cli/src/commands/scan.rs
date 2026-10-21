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

    let mut new = 0usize;
    let mut changed = 0usize;
    let mut unchanged = 0usize;
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
                        .with_detail("vbs scan"),
                )?;
                new += 1;
                bar.println(format!(
                    "  + {} ({})",
                    detected.name,
                    detected.path.display()
                ));
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    println!(
        "scanned {new} new, {changed} changed, {unchanged} unchanged across {} roots",
        roots.len()
    );
    Ok(())
}
