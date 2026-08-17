//! Desktop application context:
//! shared store + scanner + config, mirroring the CLI `App`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use vobes_config::Config;
use vobes_core::{ActivityEvent, Error, Result, Vobe, VobeId};
use vobes_scan::{DefaultScanner, Scanner};
use vobes_store::{Filter, SavedFilter, Sort, SqliteStore, Store};

/// Wires together config, store, scanner. Managed by Tauri state.
pub struct DesktopCtx {
    /// Active config — wrapped in a Mutex so the settings panel can
    /// swap it on the fly when the user changes scan settings.
    pub config: Mutex<Config>,
    /// Storage.
    pub store: Arc<dyn Store>,
    /// Scanner — wrapped in a Mutex for the same reason as `config`.
    pub scanner: Mutex<Arc<dyn Scanner>>,
    /// DB file (used by export defaults). Empty when running degraded
    /// (in-memory store).
    pub db_path: PathBuf,
    /// Single mutex to serialize mutating operations (scan, sync).
    pub scan_lock: Mutex<()>,
}

impl DesktopCtx {
    /// Load desktop context from default paths. Never fails — every
    /// recoverable error is logged and a fallback is used. Returning
    /// `Err` from the Tauri setup hook makes `tauri` itself panic,
    /// and panicking inside the `NSApplicationDidFinishLaunching`
    /// observer aborts the process instead of exiting cleanly.
    pub fn load() -> Self {
        Self::load_at(vobes_config::config_path(), vobes_config::db_path())
    }

    /// Build a context from explicit config + db paths. Used by `load()`
    /// in production and by tests with tempdir paths to exercise each
    /// branch of the fallback chain without touching the real state dir.
    fn load_at(config_path: Option<PathBuf>, db_path: Option<PathBuf>) -> Self {
        let config = Self::load_config_from(config_path.as_deref());
        match Self::open_persistent_store_at(db_path.as_deref()) {
            Some((store, db_path)) => Self::with_store(config, store, db_path),
            None => Self::degraded(config),
        }
    }

    fn load_config_from(path: Option<&Path>) -> Config {
        let Some(p) = path else {
            return Config::default();
        };
        match Config::load_from(p) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "vobes: config at {} failed to parse ({}); falling back to defaults",
                    p.display(),
                    e
                );
                Config::default()
            }
        }
    }

    fn open_persistent_store_at(path: Option<&Path>) -> Option<(Arc<dyn Store>, PathBuf)> {
        let db_path = path?;
        match SqliteStore::open(db_path) {
            Ok(s) => Some((Arc::new(s), db_path.to_path_buf())),
            Err(e) => {
                eprintln!(
                    "vobes: failed to open db at {} ({}); falling back to in-memory store",
                    db_path.display(),
                    e
                );
                None
            }
        }
    }

    fn with_store(config: Config, store: Arc<dyn Store>, db_path: PathBuf) -> Self {
        let scanner: Arc<dyn Scanner> = Arc::new(
            DefaultScanner::with_standard_detectors()
                .with_extra_excludes(config.scan.exclude.clone())
                .with_max_depth(config.scan.max_depth)
                .with_follow_symlinks(config.scan.follow_symlinks),
        );
        Self {
            config: Mutex::new(config),
            store,
            scanner: Mutex::new(scanner),
            db_path,
            scan_lock: Mutex::new(()),
        }
    }

    /// Last-resort context — in-memory store, default scanner. Reached
    /// when both the persistent SQLite path and the in-memory fallback
    /// fail to initialize.
    fn degraded(config: Config) -> Self {
        let store: Arc<dyn Store> = match SqliteStore::open_in_memory() {
            Ok(s) => Arc::new(s),
            Err(e) => {
                eprintln!(
                    "vobes: in-memory store failed to init ({}); commands will return errors",
                    e
                );
                Arc::new(BrokenStore)
            }
        };
        let scanner: Arc<dyn Scanner> = Arc::new(DefaultScanner::with_standard_detectors());
        Self {
            config: Mutex::new(config),
            store,
            scanner: Mutex::new(scanner),
            db_path: PathBuf::new(),
            scan_lock: Mutex::new(()),
        }
    }

    /// Snapshot of the current config (cheap clone).
    pub fn config_snapshot(&self) -> Config {
        self.config.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Read the current scanner (clones the Arc — cheap).
    pub fn scanner_snapshot(&self) -> Arc<dyn Scanner> {
        self.scanner
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| Arc::new(DefaultScanner::with_standard_detectors()))
    }
}

/// `Store` impl that returns errors for every operation. Reached only
/// when both the persistent and in-memory SQLite stores fail to open —
/// keeps the UI responsive so the user can see the error instead of
/// having the process abort.
struct BrokenStore;

impl Store for BrokenStore {
    fn upsert_vobe(&self, _: &Vobe) -> Result<()> {
        Err(broken())
    }
    fn get_vobe(&self, _: &VobeId) -> Result<Option<Vobe>> {
        Err(broken())
    }
    fn get_vobe_by_name(&self, _: &str) -> Result<Option<Vobe>> {
        Err(broken())
    }
    fn get_vobe_by_path(&self, _: &Path) -> Result<Option<Vobe>> {
        Err(broken())
    }
    fn list_vobes(&self, _: &Filter, _: Sort) -> Result<Vec<Vobe>> {
        Err(broken())
    }
    fn delete_vobe(&self, _: &VobeId) -> Result<()> {
        Err(broken())
    }
    fn record_activity(&self, _: &ActivityEvent) -> Result<()> {
        Err(broken())
    }
    fn recent_activity(&self, _: usize) -> Result<Vec<ActivityEvent>> {
        Err(broken())
    }
    fn vobe_activity(&self, _: &VobeId, _: usize) -> Result<Vec<ActivityEvent>> {
        Err(broken())
    }
    fn export_json(&self, _: &Path) -> Result<()> {
        Err(broken())
    }
    fn import_json(&self, _: &Path) -> Result<()> {
        Err(broken())
    }
    fn purge_all(&self) -> Result<()> {
        Err(broken())
    }
    fn list_saved_filters(&self) -> Result<Vec<SavedFilter>> {
        Err(broken())
    }
    fn upsert_saved_filter(&self, _: &SavedFilter) -> Result<()> {
        Err(broken())
    }
    fn delete_saved_filter(&self, _: &str) -> Result<()> {
        Err(broken())
    }
}

fn broken() -> Error {
    Error::storage("storage unavailable — database failed to initialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use vobes_core::ActivityKind;

    fn temp_dir(label: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "vobes-ctx-test-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn load_config_from_none_returns_default() {
        let cfg = DesktopCtx::load_config_from(None);
        assert_eq!(
            serde_json::to_string(&cfg).unwrap(),
            serde_json::to_string(&Config::default()).unwrap()
        );
    }

    #[test]
    fn load_config_from_missing_path_returns_default() {
        let dir = temp_dir("missing");
        let cfg_path = dir.join("does-not-exist.toml");
        assert!(!cfg_path.exists());
        let cfg = DesktopCtx::load_config_from(Some(&cfg_path));
        assert_eq!(
            serde_json::to_string(&cfg).unwrap(),
            serde_json::to_string(&Config::default()).unwrap()
        );
        cleanup(&dir);
    }

    #[test]
    fn load_config_from_garbage_returns_default() {
        let dir = temp_dir("garbage");
        let cfg_path = dir.join("config.toml");
        std::fs::write(&cfg_path, b"this is not valid toml = = =").unwrap();
        let cfg = DesktopCtx::load_config_from(Some(&cfg_path));
        assert_eq!(
            serde_json::to_string(&cfg).unwrap(),
            serde_json::to_string(&Config::default()).unwrap()
        );
        cleanup(&dir);
    }

    #[test]
    fn open_persistent_store_at_none_returns_none() {
        assert!(DesktopCtx::open_persistent_store_at(None).is_none());
    }

    #[test]
    fn open_persistent_store_at_valid_dir_succeeds() {
        let dir = temp_dir("valid-db");
        let db_path = dir.join("vobes.db");
        let (store, returned) =
            DesktopCtx::open_persistent_store_at(Some(&db_path)).expect("store should open");
        assert_eq!(returned, db_path);
        assert!(store.list_vobes(&Filter::all(), Sort::Name).is_ok());
        cleanup(&dir);
    }

    #[test]
    fn open_persistent_store_at_unwritable_path_returns_none() {
        let bad = PathBuf::from("/dev/null/vobes-should-not-exist.db");
        assert!(DesktopCtx::open_persistent_store_at(Some(&bad)).is_none());
    }

    #[test]
    fn load_at_no_paths_uses_degraded() {
        let ctx = DesktopCtx::load_at(None, None);
        assert!(ctx.db_path.as_os_str().is_empty());
        let snap = ctx.config_snapshot();
        assert_eq!(
            serde_json::to_string(&snap).unwrap(),
            serde_json::to_string(&Config::default()).unwrap()
        );
    }

    #[test]
    fn load_at_valid_paths_uses_persistent() {
        let dir = temp_dir("happy");
        let cfg = dir.join("config.toml");
        let db = dir.join("vobes.db");
        let ctx = DesktopCtx::load_at(Some(cfg.clone()), Some(db.clone()));
        assert_eq!(ctx.db_path, db);
        let v = Vobe::new("happy-path", &db);
        let id = v.id.clone();
        ctx.store.upsert_vobe(&v).expect("upsert");
        let fetched = ctx.store.get_vobe(&id).expect("get").expect("present");
        assert_eq!(fetched.name, "happy-path");
        cleanup(&dir);
    }

    #[test]
    fn load_at_invalid_config_still_loads_persistent() {
        let dir = temp_dir("bad-cfg");
        let cfg = dir.join("config.toml");
        let db = dir.join("vobes.db");
        std::fs::write(&cfg, b"!!! not toml !!!").unwrap();
        let ctx = DesktopCtx::load_at(Some(cfg.clone()), Some(db.clone()));
        let snap = ctx.config_snapshot();
        assert_eq!(
            serde_json::to_string(&snap).unwrap(),
            serde_json::to_string(&Config::default()).unwrap()
        );
        assert_eq!(ctx.db_path, db);
        cleanup(&dir);
    }

    #[test]
    fn broken_store_returns_err_on_every_op() {
        let s: Arc<dyn Store> = Arc::new(BrokenStore);
        let id = VobeId::from_string("vobe_x");
        let v = Vobe::new("x", Path::new("/tmp"));
        let act = ActivityEvent::now(id.clone(), ActivityKind::Created);
        let filter = SavedFilter {
            id: "f".into(),
            label: "f".into(),
            query: "is:pinned".into(),
            created_at: Utc::now(),
        };
        let tmp = PathBuf::from("/tmp");

        assert!(s.upsert_vobe(&v).is_err());
        assert!(s.get_vobe(&id).is_err());
        assert!(s.get_vobe_by_name("x").is_err());
        assert!(s.get_vobe_by_path(&tmp).is_err());
        assert!(s.list_vobes(&Filter::all(), Sort::Name).is_err());
        assert!(s.delete_vobe(&id).is_err());
        assert!(s.record_activity(&act).is_err());
        assert!(s.recent_activity(10).is_err());
        assert!(s.vobe_activity(&id, 10).is_err());
        assert!(s.export_json(&tmp).is_err());
        assert!(s.import_json(&tmp).is_err());
        assert!(s.purge_all().is_err());
        assert!(s.list_saved_filters().is_err());
        assert!(s.upsert_saved_filter(&filter).is_err());
        assert!(s.delete_saved_filter("f").is_err());
    }
}
