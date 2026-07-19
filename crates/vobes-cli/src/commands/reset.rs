//! `vbs reset` — purge all vobes + activity, then re-scan from scratch.
//!
//! Dangerous and irreversible. Prompts for confirmation unless
//! `--yes` is passed. Stale vobes (e.g. old `src-tauri` entries) persist
//! until this runs, because normal scan only adds.

use console::style;
use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::commands::shared::{absolute_normalized, vobe_from_detection};
use vobes_cli::app::App;

pub fn run(app: &App, yes: bool) -> Result<()> {
    if !yes {
        println!(
            "{}",
            style("This will DELETE every vobe and all activity. No undo.").red()
        );
        print!("Type 'yes' to continue: ");
        use std::io::Write;
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        if input.trim() != "yes" {
            println!("aborted.");
            return Ok(());
        }
    }

    app.store.purge_all()?;

    let roots = app.config.resolved_roots();
    let mut found = 0usize;
    for root in &roots {
        if !root.exists() {
            continue;
        }
        let pairs = app.scanner.scan(root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            let vobe = vobe_from_detection(&path, &detection)?;
            app.store.upsert_vobe(&vobe)?;
            app.store.record_activity(
                &ActivityEvent::now(vobe.id.clone(), ActivityKind::Scanned)
                    .with_detail("vbs reset"),
            )?;
            found += 1;
        }
    }

    println!("reset complete: {found} vobes re-discovered");
    Ok(())
}
