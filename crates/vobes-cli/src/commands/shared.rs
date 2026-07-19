//! Shared helpers used across CLI subcommands.

use std::path::{Path, PathBuf};
use vobes_core::{normalize, Result, Vobe};
use vobes_scan::Detection;

use vobes_cli::app::App;

/// Make a path absolute against the current working directory, then
/// normalize separators/dots. Stable across platforms and input styles.
pub fn absolute_normalized(path: &Path) -> PathBuf {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    normalize(&abs)
}

/// Build a `Vobe` from a `(path, detection)` pair, pulling git info if
/// the detection reports a repo.
pub fn vobe_from_detection(path: &Path, detection: &Detection) -> Result<Vobe> {
    let path = absolute_normalized(path);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();
    let mut vobe = Vobe::new(name, &path);
    vobe.framework = detection.framework.clone();
    vobe.language = detection.language.clone();
    vobe.package_manager = detection.package_manager.clone();
    vobe.touch_modified();
    if detection.is_repo {
        if let Some(git) = vobes_git::read_git_info(&path)? {
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
    let abs = absolute_normalized(Path::new(name_or_path));
    if let Some(v) = app.store.get_vobe_by_path(&abs)? {
        return Ok(Some(v));
    }
    Ok(None)
}
