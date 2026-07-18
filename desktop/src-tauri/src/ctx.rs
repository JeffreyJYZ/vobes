//! Desktop application context:
//! shared store + scanner + config, mirroring the CLI `App`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use vobes_config::Config;
use vobes_scan::{DefaultScanner, Scanner};
use vobes_store::{SqliteStore, Store};

/// Wires together config, store, scanner. Managed by Tauri state.
pub struct DesktopCtx {
    /// Loaded config.
    pub config: Config,
    /// Storage.
    pub store: Arc<dyn Store>,
    /// Scanner.
    pub scanner: Arc<dyn Scanner>,
    /// DB file (used by export defaults).
    pub db_path: PathBuf,
    /// Single mutex to serialize mutating operations (scan, sync).
    pub scan_lock: Mutex<()>,
}

impl DesktopCtx {
    /// Load desktop context from default paths.
    pub fn load() -> vobes_core::Result<Self> {
        let cfg_path = vobes_config::config_path();
        let config = match &cfg_path {
            Some(p) => {
                Config::load_from(p).map_err(|e| vobes_core::Error::config(e.to_string()))?
            }
            None => Config::default(),
        };

        let db_path = vobes_config::db_path().ok_or_else(|| {
            vobes_core::Error::internal("cannot resolve platform state dir for db")
        })?;

        let store: Arc<dyn Store> = Arc::new(SqliteStore::open(&db_path)?);

        let scanner: Arc<dyn Scanner> = Arc::new(
            DefaultScanner::with_standard_detectors()
                .with_extra_excludes(config.scan.exclude.clone())
                .with_max_depth(config.scan.max_depth)
                .with_follow_symlinks(config.scan.follow_symlinks),
        );

        Ok(Self {
            config,
            store,
            scanner,
            db_path,
            scan_lock: Mutex::new(()),
        })
    }
}
