//! Wires together config, store, scanner into a single app context.

use std::path::PathBuf;
use std::sync::Arc;

use vobes_config::{config_path, Config};
use vobes_scan::{DefaultScanner, Scanner};
use vobes_store::{SqliteStore, Store};

/// Top-level CLI context.
pub struct App {
    /// Loaded configuration.
    pub config: Config,
    /// Storage surface.
    pub store: Arc<dyn Store>,
    /// Scanner (DefaultScanner with config-driven excludes + depth).
    pub scanner: Arc<dyn Scanner>,
    /// Path to the active config file (for `init` and messages).
    pub config_path: Option<PathBuf>,
}

impl App {
    /// Load the app: config from default path, store from default db
    /// path, scanner with config-driven excludes.
    pub fn load() -> vobes_core::Result<Self> {
        let cfg_path = config_path();
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
            config_path: cfg_path,
        })
    }
}
