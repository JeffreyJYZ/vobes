//! `vbs scan` — discover projects in configured roots.

use std::collections::HashSet;

use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::app::App;
use crate::commands::shared::{absolute_normalized, vobe_from_detection};

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

    let mut found = 0usize;
    let mut new_ids: HashSet<String> = HashSet::new();
    let mut total_dirs_skipped = 0usize;
    let mut total_dirs_walked = 0usize;
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
            // Skip if already tracked by path
            if app.store.get_vobe_by_path(&path)?.is_some() {
                continue;
            }
            let mut vobe = vobe_from_detection(&path, &detection)?;
            app.store.upsert_vobe(&vobe)?;
            new_ids.insert(vobe.id.as_str().to_string());
            app.store.record_activity(
                &ActivityEvent::now(vobe.id.clone(), ActivityKind::Scanned).with_detail("vbs scan"),
            )?;
            vobe.touch_modified();
            found += 1;
            bar.println(format!("  + {} ({})", vobe.name, vobe.path.display()));
        }
        let _ = &mut total_dirs_skipped;
        let _ = &mut total_dirs_walked;
        bar.inc(1);
    }
    bar.finish_and_clear();

    println!("discovered {found} vobes across {} roots", roots.len());
    Ok(())
}
