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
        let config = Self::load_config();
        match Self::open_persistent_store() {
            Some((store, db_path)) => Self::with_store(config, store, db_path),
            None => Self::degraded(config),
        }
    }

    fn load_config() -> Config {
        match vobes_config::config_path() {
            Some(p) => match Config::load_from(&p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "vobes: config at {} failed to parse ({}); falling back to defaults",
                        p.display(),
                        e
                    );
                    Config::default()
                }
            },
            None => Config::default(),
        }
    }

    fn open_persistent_store() -> Option<(Arc<dyn Store>, PathBuf)> {
        let db_path = match vobes_config::db_path() {
            Some(p) => p,
            None => {
                eprintln!(
                    "vobes: cannot resolve platform state dir; falling back to in-memory store"
                );
                return None;
            }
        };
        match SqliteStore::open(&db_path) {
            Ok(s) => Some((Arc::new(s), db_path)),
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
