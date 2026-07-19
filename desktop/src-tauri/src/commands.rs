//! Tauri command handlers — same core, just exposed to the frontend.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::State;

use vobes_core::{ActivityEvent, ActivityKind, Result};
use vobes_store::{Filter, Sort};

use crate::commands::shared::{absolute_normalized, lookup_vobe, vobe_from_detection};
use crate::ctx::DesktopCtx;
use crate::dto::{ActivityDto, VobeDto};

/// List all tracked vobes (excluding archived).
#[tauri::command]
pub async fn list_vobes(state: State<'_, Arc<DesktopCtx>>) -> Result<Vec<VobeDto>> {
    let vobes = state
        .store
        .list_vobes(&Filter::all().exclude_archived(), Sort::LastModified)?;
    Ok(vobes.iter().map(VobeDto::from).collect())
}

/// Fetch a single vobe by name or path.
#[tauri::command]
pub async fn get_vobe(state: State<'_, Arc<DesktopCtx>>, name: String) -> Result<Option<VobeDto>> {
    Ok(lookup_vobe(&state.store, &name)?.map(|v| VobeDto::from(&v)))
}

/// Recent activity across all vobes.
#[tauri::command]
pub async fn recent_activity(
    state: State<'_, Arc<DesktopCtx>>,
    limit: usize,
) -> Result<Vec<ActivityDto>> {
    let events = state.store.recent_activity(limit)?;
    Ok(events.iter().map(ActivityDto::from).collect())
}

/// Activity for a single vobe.
#[tauri::command]
pub async fn vobe_activity(
    state: State<'_, Arc<DesktopCtx>>,
    vobe_id: String,
    limit: usize,
) -> Result<Vec<ActivityDto>> {
    let id = vobes_core::VobeId::from_string(vobe_id);
    let events = state.store.vobe_activity(&id, limit)?;
    Ok(events.iter().map(ActivityDto::from).collect())
}

/// Scan configured roots, add newly discovered vobes.
#[tauri::command]
pub async fn scan(state: State<'_, Arc<DesktopCtx>>) -> Result<usize> {
    let _guard = state
        .scan_lock
        .lock()
        .map_err(|e| vobes_core::Error::internal(format!("scan lock: {e}")))?;
    let mut found = 0usize;
    for root in state.config.resolved_roots() {
        if !root.exists() {
            continue;
        }
        let pairs = state.scanner.scan(&root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            if state.store.get_vobe_by_path(&path)?.is_some() {
                continue;
            }
            let vobe = vobe_from_detection(&path, &detection)?;
            state.store.upsert_vobe(&vobe)?;
            state.store.record_activity(
                &ActivityEvent::now(vobe.id.clone(), ActivityKind::Scanned)
                    .with_detail("desktop scan"),
            )?;
            found += 1;
        }
    }
    Ok(found)
}

/// Sync: re-scan roots, refresh existing vobes, add new ones.
#[tauri::command]
pub async fn sync(state: State<'_, Arc<DesktopCtx>>) -> Result<(usize, usize)> {
    let _guard = state
        .scan_lock
        .lock()
        .map_err(|e| vobes_core::Error::internal(format!("sync lock: {e}")))?;
    let mut added = 0usize;
    let mut updated = 0usize;
    for root in state.config.resolved_roots() {
        if !root.exists() {
            continue;
        }
        let pairs = state.scanner.scan(&root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            let existing = state.store.get_vobe_by_path(&path)?;
            let mut vobe = vobe_from_detection(&path, &detection)?;
            if let Some(prev) = existing {
                vobe.id = prev.id;
                vobe.created_at = prev.created_at;
                vobe.tags = prev.tags;
                vobe.notes = prev.notes;
                vobe.metadata = prev.metadata;
                vobe.last_opened = prev.last_opened;
                vobe.touch_modified();
                state.store.upsert_vobe(&vobe)?;
                updated += 1;
            } else {
                state.store.upsert_vobe(&vobe)?;
                state.store.record_activity(
                    &ActivityEvent::now(vobe.id.clone(), ActivityKind::Scanned)
                        .with_detail("desktop sync"),
                )?;
                added += 1;
            }
        }
    }
    Ok((added, updated))
}

/// Manually add a vobe for a path.
#[tauri::command]
pub async fn add_vobe(state: State<'_, Arc<DesktopCtx>>, path: String) -> Result<VobeDto> {
    let abs = absolute_normalized(Path::new(&path));
    if !abs.exists() {
        return Err(vobes_core::Error::not_found(abs.display().to_string()));
    }
    if let Some(existing) = state.store.get_vobe_by_path(&abs)? {
        return Ok(VobeDto::from(&existing));
    }
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
    state.store.upsert_vobe(&vobe)?;
    state.store.record_activity(
        &ActivityEvent::now(vobe.id.clone(), ActivityKind::Created).with_detail("desktop add"),
    )?;
    Ok(VobeDto::from(&vobe))
}

/// Remove a vobe.
#[tauri::command]
pub async fn remove_vobe(state: State<'_, Arc<DesktopCtx>>, name: String) -> Result<()> {
    let Some(vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    state.store.delete_vobe(&vobe.id)?;
    Ok(())
}

/// Mark opened + record activity (editor launch handled by frontend via shell plugin).
#[tauri::command]
pub async fn open_vobe(state: State<'_, Arc<DesktopCtx>>, name: String) -> Result<()> {
    let Some(mut vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    vobe.touch_opened();
    state.store.upsert_vobe(&vobe)?;
    state.store.record_activity(
        &ActivityEvent::now(vobe.id.clone(), ActivityKind::Opened).with_detail("desktop open"),
    )?;
    Ok(())
}

/// Export all data as JSON.
#[tauri::command]
pub async fn export_json(state: State<'_, Arc<DesktopCtx>>, out: Option<String>) -> Result<String> {
    let path = match out {
        Some(p) => PathBuf::from(p),
        None => {
            let base = vobes_config::snapshots_dir().unwrap_or_else(|| PathBuf::from("."));
            std::fs::create_dir_all(&base).ok();
            let ts = chrono::Utc::now().format("%Y-%m-%d-%H%M%S");
            base.join(format!("vobes-{ts}.json"))
        }
    };
    state.store.export_json(&path)?;
    Ok(path.to_string_lossy().to_string())
}

/// Shared helpers reused from CLI logic. Kept private to this module.
pub mod shared {
    use std::path::{Path, PathBuf};
    use vobes_core::{normalize, Result, Vobe};
    use vobes_scan::Detection;
    use vobes_store::Store;

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
    pub fn lookup_vobe(store: &dyn Store, name_or_path: &str) -> Result<Option<Vobe>> {
        if let Some(v) = store.get_vobe_by_name(name_or_path)? {
            return Ok(Some(v));
        }
        let p = std::path::PathBuf::from(name_or_path);
        let abs = if p.is_absolute() {
            p
        } else {
            std::env::current_dir().unwrap_or_default().join(p)
        };
        if let Some(v) = store.get_vobe_by_path(&abs)? {
            return Ok(Some(v));
        }
        Ok(None)
    }
}
