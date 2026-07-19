//! `vbs add <path>` — manually track a vobe.

use std::path::Path;

use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::app::App;
use crate::commands::shared::{absolute_normalized, vobe_from_detection};

pub fn run(app: &App, path: &str) -> Result<()> {
    let abs = absolute_normalized(Path::new(path));
    if !abs.exists() {
        return Err(vobes_core::Error::not_found(abs.display().to_string()));
    }
    if let Some(existing) = app.store.get_vobe_by_path(&abs)? {
        println!("already tracked as {}", existing.name);
        return Ok(());
    }

    // Run detectors on the path
    let mut detection = vobes_scan::Detection::empty();
    let detectors: Vec<Box<dyn vobes_scan::Detector>> = vec![
        Box::new(vobes_scan::RepoDetector::new()),
        Box::new(vobes_scan::LanguageDetector::new()),
        Box::new(vobes_scan::PackageManagerDetector::new()),
        Box::new(vobes_scan::FrameworkDetector::new()),
    ];
    for d in &detectors {
        if let Ok(Some(det)) = d.detect(&abs) {
            detection.merge(det);
        }
    }

    let vobe = vobe_from_detection(&abs, &detection)?;
    app.store.upsert_vobe(&vobe)?;
    app.store.record_activity(
        &ActivityEvent::now(vobe.id.clone(), ActivityKind::Created).with_detail("vbs add"),
    )?;
    println!("added: {} -> {}", vobe.name, vobe.path.display());
    Ok(())
}
