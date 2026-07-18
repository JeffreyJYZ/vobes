//! Shared helpers used across CLI subcommands.

use std::path::Path;
use vobes_core::{Result, Vobe};
use vobes_scan::Detection;

use crate::app::App;

/// Build a `Vobe` from a `(path, detection)` pair, pulling git info if
/// the detection reports a repo.
pub fn vobe_from_detection(path: &Path, detection: &Detection) -> Result<Vobe> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();
    let mut vobe = Vobe::new(name, path.to_path_buf());
    vobe.framework = detection.framework.clone();
    vobe.language = detection.language.clone();
    vobe.package_manager = detection.package_manager.clone();
    vobe.touch_modified();
    if detection.is_repo {
        if let Some(git) = vobes_git::read_git_info(path)? {
            vobe = vobe.with_git(git);
        }
    }
    Ok(vobe)
}

/// Resolve a name argument that could be either a vobe name or a path.
/// Returns the vobe if found.
pub fn lookup_vobe(app: &App, name_or_path: &str) -> Result<Option<Vobe>> {
    // Try as name first
    if let Some(v) = app.store.get_vobe_by_name(name_or_path)? {
        return Ok(Some(v));
    }
    // Try as path
    let p = std::path::PathBuf::from(name_or_path);
    let abs = if p.is_absolute() {
        p
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };
    if let Some(v) = app.store.get_vobe_by_path(&abs)? {
        return Ok(Some(v));
    }
    Ok(None)
}
