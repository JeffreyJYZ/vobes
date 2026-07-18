//! Configuration model with sensible defaults.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// General workspace settings.
    pub general: GeneralConfig,
    /// Scanner settings.
    pub scan: ScanConfig,
    /// Display settings.
    pub display: DisplayConfig,
    /// Git settings.
    pub git: GitConfig,
    /// Export settings.
    pub export: ExportConfig,
}

/// General workspace metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneralConfig {
    /// Human-friendly workspace name.
    pub name: Option<String>,
}

/// Scanner settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScanConfig {
    /// Roots to scan. `~` expanded.
    pub roots: Vec<String>,
    /// Additional excludes on top of the built-in defaults.
    pub exclude: Vec<String>,
    /// Max directory depth to walk.
    pub max_depth: usize,
    /// Whether to follow symlinks.
    pub follow_symlinks: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            roots: vec!["~/dev".to_string()],
            exclude: Vec::new(),
            max_depth: 4,
            follow_symlinks: false,
        }
    }
}

/// Display preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DisplayConfig {
    /// `"auto"`, `"light"`, `"dark"`.
    pub theme: String,
    /// `"relative"` (e.g., "3 hours ago") or `"absolute"`.
    pub date_format: String,
    /// Default sort field: `"last_modified"`, `"name"`, `"last_opened"`, `"created_at"`.
    pub default_sort: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            date_format: "relative".to_string(),
            default_sort: "last_modified".to_string(),
        }
    }
}

/// Git settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GitConfig {
    /// How long cached git state is considered fresh, in seconds.
    pub cache_ttl_seconds: u64,
    /// Whether to auto-fetch upstream on sync.
    pub fetch_upstream: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 60,
            fetch_upstream: false,
        }
    }
}

/// Export settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExportConfig {
    /// Where JSON snapshots are written. `~` expanded.
    pub path: String,
    /// Export format. Currently `"json"`.
    pub format: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            path: "~/.vobes/snapshots".to_string(),
            format: "json".to_string(),
        }
    }
}

impl Config {
    /// Load config from a TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Load config from the given file path. If the file does not exist,
    /// returns the default config.
    pub fn load_from(path: &std::path::Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Read(path.to_path_buf(), e.to_string()))?;
        Self::from_toml_str(&s).map_err(|e| ConfigError::Parse(path.to_path_buf(), e.to_string()))
    }

    /// Resolve scan roots to absolute paths, expanding `~`.
    pub fn resolved_roots(&self) -> Vec<PathBuf> {
        self.scan
            .roots
            .iter()
            .map(|r| expand_home(r).unwrap_or_else(|| PathBuf::from(r)))
            .collect()
    }
}

/// Expand a leading `~` to the user's home directory.
pub fn expand_home(p: &str) -> Option<PathBuf> {
    if p == "~" || p.starts_with("~/") {
        let home = dirs::home_dir()?;
        let tail = &p[1..];
        Some(home.join(tail.trim_start_matches('/')))
    } else {
        Some(PathBuf::from(p))
    }
}

/// Config-level error.
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    /// File could not be read.
    #[error("cannot read config {0}: {1}")]
    Read(PathBuf, String),
    /// File could not be parsed.
    #[error("cannot parse config {0}: {1}")]
    Parse(PathBuf, String),
}
