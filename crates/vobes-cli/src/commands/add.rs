//! `vbs add <path>` — manually track a vobe.

use std::path::Path;

use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::commands::shared::{absolute_normalized, vobe_from_detection};
use vobes_cli::app::App;

pub fn run(app: &App, path: &str) -> Result<()> {
    let abs = absolute_normalized(Path::new(path));
    if !abs.exists() {
        return Err(vobes_core::Error::not_found(abs.display().to_string()));
    }
    if let Some(existing) = app.store.get_vobe_by_path(&abs)? {
        println!("already tracked as {}", existing.name);
        return Ok(());
    }

    let detection = app.scanner.detect(&abs)?;
    let vobe = vobe_from_detection(&abs, &detection)?;
    app.store.upsert_vobe(&vobe)?;
    app.store.record_activity(
        &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Created).with_detail("vbs add"),
    )?;
    println!("added: {} -> {}", vobe.name, vobe.path.display());
    Ok(())
}
