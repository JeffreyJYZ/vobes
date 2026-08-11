//! Tauri command handlers — same core, just exposed to the frontend.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use vobes_core::{ActivityEvent, ActivityKind, Result};
use vobes_store::{Filter, SavedFilter, Sort};

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

/// Recent activity across all vobes. If `actor` is supplied, only
/// events whose `actor` label matches exactly are returned — backs
/// the activity-feed "agent vs human" filter.
#[tauri::command]
pub async fn recent_activity(
    state: State<'_, Arc<DesktopCtx>>,
    limit: usize,
    actor: Option<String>,
) -> Result<Vec<ActivityDto>> {
    let events = state
        .store
        .recent_activity_by_actor(actor.as_deref(), limit)?;
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
    let config = state.config_snapshot();
    let scanner = state.scanner_snapshot();
    let mut found = 0usize;
    for root in config.resolved_roots() {
        if !root.exists() {
            continue;
        }
        let pairs = scanner.scan(&root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            if state.store.get_vobe_by_path(&path)?.is_some() {
                continue;
            }
            let vobe = vobe_from_detection(&path, &detection)?;
            state.store.upsert_vobe(&vobe)?;
            state.store.record_activity(
                &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Scanned)
                    .with_detail("desktop scan"),
            )?;
            found += 1;
        }
    }
    Ok(found)
}

/// Dangerous: purge every vobe and all activity, then re-scan from scratch.
///
/// The frontend must confirm with the user before calling this — there is
/// no undo. Stale entries (e.g. old `src-tauri` vobes) linger until this
/// runs, since normal scan only adds, never removes.
#[tauri::command]
pub async fn reset_and_rescan(state: State<'_, Arc<DesktopCtx>>) -> Result<usize> {
    let _guard = state
        .scan_lock
        .lock()
        .map_err(|e| vobes_core::Error::internal(format!("scan lock: {e}")))?;
    let config = state.config_snapshot();
    let scanner = state.scanner_snapshot();
    state.store.purge_all()?;
    let mut found = 0usize;
    for root in config.resolved_roots() {
        if !root.exists() {
            continue;
        }
        let pairs = scanner.scan(&root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            let vobe = vobe_from_detection(&path, &detection)?;
            state.store.upsert_vobe(&vobe)?;
            state.store.record_activity(
                &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Scanned)
                    .with_detail("desktop reset+rescan"),
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
    let config = state.config_snapshot();
    let scanner = state.scanner_snapshot();
    let mut added = 0usize;
    let mut updated = 0usize;
    for root in config.resolved_roots() {
        if !root.exists() {
            continue;
        }
        let pairs = scanner.scan(&root)?;
        for (path, detection) in pairs {
            let path = absolute_normalized(&path);
            let existing = state.store.get_vobe_by_path(&path)?;
            let mut vobe = vobe_from_detection(&path, &detection)?;
            if let Some(prev) = existing {
                vobe.id = prev.id;
                vobe.created_at = prev.created_at;
                vobe.tags = prev.tags;
                vobe.notes = prev.notes;
                vobe.pinned = prev.pinned;
                vobe.metadata = prev.metadata;
                vobe.last_opened = prev.last_opened;
                vobe.touch_modified();
                state.store.upsert_vobe(&vobe)?;
                updated += 1;
            } else {
                state.store.upsert_vobe(&vobe)?;
                state.store.record_activity(
                    &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Scanned)
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
        &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Created).with_detail("desktop add"),
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
        &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Opened).with_detail("desktop open"),
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

/// Resolved platform paths the Settings view surfaces read-only.
#[derive(Debug, Serialize)]
pub struct PathsDto {
    /// Config file location.
    pub config: String,
    /// SQLite database location.
    pub db: String,
    /// JSON snapshots directory.
    pub snapshots: String,
    /// State root (parent of the above).
    pub state_dir: String,
}

fn paths_dto() -> PathsDto {
    fn s(p: Option<std::path::PathBuf>) -> String {
        p.map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default()
    }
    PathsDto {
        config: s(vobes_config::config_path()),
        db: s(vobes_config::db_path()),
        snapshots: s(vobes_config::snapshots_dir()),
        state_dir: s(vobes_config::state_dir()),
    }
}

/// Lightweight DTO mirroring the user's full config for the Settings view.
#[derive(Debug, Serialize)]
pub struct ConfigDto {
    /// Absolute path to the config file the desktop is using.
    pub path: String,
    /// Resolved platform paths (read-only).
    pub paths: PathsDto,
    /// Current config values.
    pub config: vobes_config::Config,
}

/// Return the current config + where it's loaded from.
#[tauri::command]
pub async fn get_config(state: State<'_, Arc<DesktopCtx>>) -> Result<ConfigDto> {
    let path = vobes_config::config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(ConfigDto {
        path,
        paths: paths_dto(),
        config: state.config_snapshot(),
    })
}

/// Persist a new config to disk and apply it to the running context.
///
/// The scanner and store are reconfigured on the fly so settings take
/// effect without a restart.
#[tauri::command]
pub async fn save_config(
    state: State<'_, Arc<DesktopCtx>>,
    new_config: vobes_config::Config,
) -> Result<ConfigDto> {
    let path = vobes_config::config_path()
        .ok_or_else(|| vobes_core::Error::internal("cannot resolve platform config dir"))?;
    new_config
        .save_to(&path)
        .map_err(|e| vobes_core::Error::config(e.to_string()))?;
    // Rebuild scanner with new settings. Store is shared and untouched.
    {
        let mut guard = state
            .scanner
            .lock()
            .map_err(|e| vobes_core::Error::internal(format!("scanner lock: {e}")))?;
        let scanner: Arc<dyn vobes_scan::Scanner> = Arc::new(
            vobes_scan::DefaultScanner::with_standard_detectors()
                .with_extra_excludes(new_config.scan.exclude.clone())
                .with_max_depth(new_config.scan.max_depth)
                .with_follow_symlinks(new_config.scan.follow_symlinks),
        );
        *guard = scanner;
    }
    // Persist new config into the shared context.
    {
        let mut guard = state
            .config
            .lock()
            .map_err(|e| vobes_core::Error::internal(format!("config lock: {e}")))?;
        *guard = new_config.clone();
    }
    Ok(ConfigDto {
        path: path.to_string_lossy().to_string(),
        paths: paths_dto(),
        config: new_config,
    })
}

/// Reveal the vobe's path in the OS file manager.
#[tauri::command]
pub async fn reveal_in_finder(state: State<'_, Arc<DesktopCtx>>, name: String) -> Result<()> {
    let Some(vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    crate::platform::reveal(&vobe.path)?;
    Ok(())
}

/// Copy a string to the system clipboard. No-op on the backend; the
/// frontend can also use `navigator.clipboard`, but a Tauri command
/// keeps the API uniform and avoids browser quirks in WebView2.
#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<usize> {
    Ok(text.len())
}

/// Update a vobe's `notes` field. Returns the refreshed vobe.
#[tauri::command]
pub async fn save_notes(
    state: State<'_, Arc<DesktopCtx>>,
    name: String,
    notes: Option<String>,
) -> Result<VobeDto> {
    let Some(mut vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    vobe.notes = notes.filter(|s| !s.is_empty());
    state.store.upsert_vobe(&vobe)?;
    Ok(VobeDto::from(&vobe))
}

/// Set the "pinned" flag for a vobe. Returned by `list_vobes` as a hint.
#[tauri::command]
pub async fn set_pinned(
    state: State<'_, Arc<DesktopCtx>>,
    name: String,
    pinned: bool,
) -> Result<()> {
    let Some(mut vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    if vobe.pinned != pinned {
        vobe.pinned = pinned;
        state.store.upsert_vobe(&vobe)?;
    }
    Ok(())
}

/// List the names of currently pinned vobes.
#[tauri::command]
pub async fn get_pinned(state: State<'_, Arc<DesktopCtx>>) -> Result<Vec<String>> {
    let vobes = state.store.list_vobes(&Filter::all(), Sort::LastModified)?;
    Ok(vobes
        .iter()
        .filter(|v| v.pinned)
        .map(|v| v.name.clone())
        .collect())
}

/// Update a vobe's tag set. Returns the refreshed vobe.
#[tauri::command]
pub async fn set_tags(
    state: State<'_, Arc<DesktopCtx>>,
    name: String,
    tags: Vec<String>,
) -> Result<VobeDto> {
    let Some(mut vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    let cleaned: Vec<String> = tags
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    if cleaned != vobe.tags {
        vobe.tags = cleaned;
        state.store.upsert_vobe(&vobe)?;
    }
    Ok(VobeDto::from(&vobe))
}

/// First ~120 lines of the project README, returned as plain text.
/// Empty if no README exists or it can't be read.
#[tauri::command]
pub async fn read_readme(
    state: State<'_, Arc<DesktopCtx>>,
    name: String,
) -> Result<Option<String>> {
    let Some(vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    for candidate in [
        "README.md",
        "README.markdown",
        "README",
        "readme.md",
        "Readme.md",
    ] {
        let p = vobe.path.join(candidate);
        if p.is_file() {
            let s = match std::fs::read_to_string(&p) {
                Ok(s) => s,
                Err(_) => continue,
            };
            // Take the first ~120 non-empty lines to keep payload small.
            let head: String = s.lines().take(120).collect::<Vec<_>>().join("\n");
            return Ok(Some(head));
        }
    }
    Ok(None)
}

/// A single TODO/FIXME/XXX hit in a project.
#[derive(Debug, Serialize)]
pub struct TodoHit {
    /// "TODO" / "FIXME" / "XXX".
    pub kind: String,
    /// Line number, 1-indexed.
    pub line: u32,
    /// Path relative to the vobe root.
    pub file: String,
    /// Trimmed line content.
    pub text: String,
}

/// Quick scrape of TODO-style comments across common source files.
/// Capped at 100 hits, walks at most 6 levels deep, skips heavy
/// directories. Best-effort, not a full indexer.
#[tauri::command]
pub async fn scrape_todos(state: State<'_, Arc<DesktopCtx>>, name: String) -> Result<Vec<TodoHit>> {
    let Some(vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    let mut out: Vec<TodoHit> = Vec::new();
    let skip_dirs: &[&str] = &[
        "node_modules",
        "target",
        "dist",
        "build",
        ".git",
        ".next",
        ".turbo",
        ".venv",
        "venv",
        "__pycache__",
        ".cache",
        "Pods",
        ".idea",
        ".vscode",
    ];
    let exts: &[&str] = &[
        "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "rb", "go", "java", "kt", "swift", "c",
        "h", "cc", "cpp", "hpp", "cs", "php", "sh", "zsh", "bash", "lua", "ex", "exs", "scala",
        "sql", "toml", "yaml", "yml", "md",
    ];
    walk_todo(&vobe.path, &vobe.path, skip_dirs, exts, &mut out, 0, 6);
    out.truncate(100);
    Ok(out)
}

fn walk_todo(
    root: &Path,
    dir: &Path,
    skip_dirs: &[&str],
    exts: &[&str],
    out: &mut Vec<TodoHit>,
    depth: u32,
    max_depth: u32,
) {
    if depth > max_depth || out.len() >= 100 {
        return;
    }
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && name != "." && name != ".." {
            // Allow reading dotfiles but don't recurse into them by default.
            if path.is_dir() {
                continue;
            }
        }
        if path.is_dir() {
            if skip_dirs.iter().any(|s| s == &name) {
                continue;
            }
            walk_todo(root, &path, skip_dirs, exts, out, depth + 1, max_depth);
        } else if path.is_file() {
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if !exts.contains(&ext) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            for (i, line) in content.lines().enumerate() {
                if out.len() >= 100 {
                    return;
                }
                let upper = line.to_uppercase();
                let kind = if upper.contains("FIXME") {
                    Some("FIXME")
                } else if upper.contains("TODO") {
                    Some("TODO")
                } else if upper.contains("XXX") {
                    Some("XXX")
                } else {
                    None
                };
                if let Some(k) = kind {
                    let rel = path
                        .strip_prefix(root)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| path.to_string_lossy().to_string());
                    out.push(TodoHit {
                        kind: k.to_string(),
                        line: (i + 1) as u32,
                        file: rel,
                        text: line.trim().to_string(),
                    });
                }
            }
        }
    }
}

/// Shape of a "context pack" — what `vbs context` would produce.
#[derive(Debug, Serialize)]
pub struct ContextPack {
    /// The vobe record.
    pub vobe: VobeDto,
    /// Recent activity for this vobe, newest first.
    pub activity: Vec<ActivityDto>,
    /// Top-level directory entries (capped at 50).
    pub directory: Vec<String>,
    /// When this pack was generated (RFC3339).
    pub generated_at: String,
}

/// Build a paste-ready context pack for an agent.
#[tauri::command]
pub async fn context_pack(state: State<'_, Arc<DesktopCtx>>, name: String) -> Result<ContextPack> {
    let Some(vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    let id = vobes_core::VobeId::from_string(vobe.id.as_str().to_string());
    let activity = state.store.vobe_activity(&id, 25)?;
    let mut directory: Vec<String> = Vec::new();
    if let Ok(read) = std::fs::read_dir(&vobe.path) {
        for (i, e) in read.flatten().enumerate() {
            if i >= 50 {
                break;
            }
            let p = e.path();
            let rel = p
                .strip_prefix(&vobe.path)
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or_else(|_| p.to_string_lossy().to_string());
            directory.push(rel);
        }
    }
    Ok(ContextPack {
        vobe: VobeDto::from(&vobe),
        activity: activity.iter().map(ActivityDto::from).collect(),
        directory,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Open a path in the OS default app via tauri-plugin-opener.
/// Used for "open file" actions from the README or context pack.
#[tauri::command]
pub async fn open_path_external(app: tauri::AppHandle, path: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(vobes_core::Error::not_found(path));
    }
    app.opener()
        .open_path(p.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| vobes_core::Error::internal(format!("opener: {e}")))?;
    Ok(())
}

/// Saved-filter DTO — mirrors `vobes_store::SavedFilter` for IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedFilterDto {
    pub id: String,
    pub label: String,
    pub query: String,
    /// RFC3339 timestamp the filter was created.
    pub created_at: String,
}

impl From<&vobes_store::SavedFilter> for SavedFilterDto {
    fn from(f: &vobes_store::SavedFilter) -> Self {
        Self {
            id: f.id.clone(),
            label: f.label.clone(),
            query: f.query.clone(),
            created_at: f.created_at.to_rfc3339(),
        }
    }
}

/// List every saved filter, newest first.
#[tauri::command]
pub async fn list_saved_filters(state: State<'_, Arc<DesktopCtx>>) -> Result<Vec<SavedFilterDto>> {
    let filters = state.store.list_saved_filters()?;
    Ok(filters.iter().map(SavedFilterDto::from).collect())
}

/// Create or update a saved filter (id is the caller-assigned stable key).
#[tauri::command]
pub async fn save_saved_filter(
    state: State<'_, Arc<DesktopCtx>>,
    id: String,
    label: String,
    query: String,
    created_at: Option<String>,
) -> Result<SavedFilterDto> {
    let created_at = match created_at {
        Some(s) => chrono::DateTime::parse_from_rfc3339(&s)
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        None => chrono::Utc::now(),
    };
    let filter = SavedFilter {
        id,
        label,
        query,
        created_at,
    };
    state.store.upsert_saved_filter(&filter)?;
    Ok(SavedFilterDto::from(&filter))
}

/// Delete a saved filter by id.
#[tauri::command]
pub async fn remove_saved_filter(state: State<'_, Arc<DesktopCtx>>, id: String) -> Result<()> {
    state.store.delete_saved_filter(&id)
}

/// Snapshot file on disk — name, size, mtime. Listed by the
/// Settings "Snapshots" card so the user can restore or delete
/// past exports without dropping to a shell.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SnapshotInfoDto {
    /// Absolute path to the `.json` snapshot.
    pub path: String,
    /// Filename only (e.g. `vobes-2026-08-09-120000.json`).
    pub name: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last modified time, RFC3339.
    pub modified_at: String,
}

/// List every `*.json` snapshot in the platform snapshots dir,
/// newest first. Returns an empty vec when the dir does not exist
/// yet (e.g. first launch before any export).
#[tauri::command]
pub async fn list_snapshots() -> Result<Vec<SnapshotInfoDto>> {
    let dir = vobes_config::snapshots_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let meta = match e.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let modified = meta
                .modified()
                .ok()
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Utc> = t.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_default();
            out.push(SnapshotInfoDto {
                path: p.to_string_lossy().to_string(),
                name: p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                size_bytes: meta.len(),
                modified_at: modified,
            });
        }
    }
    out.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(out)
}

/// Restore (import) a snapshot file by absolute path. Replaces the
/// current store contents with the snapshot's vobes + activity +
/// saved filters. Frontend should refresh after.
#[tauri::command]
pub async fn restore_snapshot(state: State<'_, Arc<DesktopCtx>>, path: String) -> Result<()> {
    state.store.import_json(Path::new(&path))
}

/// Delete a snapshot file from the snapshots dir.
#[tauri::command]
pub async fn delete_snapshot(path: String) -> Result<()> {
    std::fs::remove_file(Path::new(&path))
        .map_err(|e| vobes_core::Error::storage(format!("delete snapshot: {e}")))
}

/// Terminal app descriptor surfaced to the frontend for the
/// "Open in terminal" selector on the Projects detail view.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TerminalAppDto {
    pub id: String,
    pub label: String,
    pub is_default: bool,
}

impl From<&crate::platform::TerminalApp> for TerminalAppDto {
    fn from(t: &crate::platform::TerminalApp) -> Self {
        Self {
            id: t.id.clone(),
            label: t.label.clone(),
            is_default: t.is_default,
        }
    }
}

/// Editor app descriptor surfaced to the frontend for the
/// "Open in editor" selector on the Projects detail view.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EditorAppDto {
    pub id: String,
    pub label: String,
    pub is_default: bool,
}

impl From<&crate::platform::EditorApp> for EditorAppDto {
    fn from(e: &crate::platform::EditorApp) -> Self {
        Self {
            id: e.id.clone(),
            label: e.label.clone(),
            is_default: e.is_default,
        }
    }
}

/// List terminal emulators installed on this machine.
#[tauri::command]
pub async fn list_terminals() -> Result<Vec<TerminalAppDto>> {
    Ok(crate::platform::list_terminals()
        .iter()
        .map(TerminalAppDto::from)
        .collect())
}

/// List editors installed on this machine.
#[tauri::command]
pub async fn list_editors() -> Result<Vec<EditorAppDto>> {
    Ok(crate::platform::list_editors()
        .iter()
        .map(EditorAppDto::from)
        .collect())
}

/// Spawn the named terminal at the vobe's path. If `app` is `None`,
/// falls back to the platform default.
#[tauri::command]
pub async fn open_in_terminal(
    state: State<'_, Arc<DesktopCtx>>,
    name: String,
    app: Option<String>,
) -> Result<()> {
    let Some(vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    crate::platform::open_terminal_with(&vobe.path, app.as_deref())?;
    state.store.record_activity(
        &ActivityEvent::now_env(vobe.id, ActivityKind::Opened).with_detail("terminal"),
    )?;
    Ok(())
}

/// Open the vobe's directory in the selected editor. `app` is an id
/// returned by `list_editors`; `None` falls back to the platform
/// default. The old frontend-side shell-plugin path is deprecated —
/// all editor launches funnel here so actor attribution works.
#[tauri::command]
pub async fn open_in_editor(
    state: State<'_, Arc<DesktopCtx>>,
    name: String,
    app: Option<String>,
) -> Result<()> {
    let Some(vobe) = lookup_vobe(&state.store, &name)? else {
        return Err(vobes_core::Error::not_found(name));
    };
    crate::platform::open_editor(&vobe.path, app.as_deref())?;
    state.store.record_activity(
        &ActivityEvent::now_env(vobe.id.clone(), ActivityKind::Opened).with_detail("editor"),
    )?;
    Ok(())
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
