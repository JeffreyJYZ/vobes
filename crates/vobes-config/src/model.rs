//! Configuration model with sensible defaults.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Top-level config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Scanner settings.
    pub scan: ScanConfig,
    /// Display settings.
    pub display: DisplayConfig,
    /// Desktop-specific preferences (ignored by the CLI).
    pub desktop: DesktopConfig,
}

/// Desktop app preferences. The CLI reads/writes this section as
/// opaque bytes; only the desktop UI surfaces the fields. Lives in
/// the same `config.toml` as the rest, so dotfile-sync just works.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DesktopConfig {
    /// Surface an OS notification when a vobe is behind upstream.
    pub notify_behind: bool,
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
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
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

    /// Serialize to a stable, commented TOML string.
    pub fn to_toml_string(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self).map_err(|e| ConfigError::Write(e.to_string()))
    }

    /// Persist the config to disk, creating parent dirs as needed.
    pub fn save_to(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::Write(format!("mkdir {}: {e}", parent.display())))?;
        }
        let body = self.to_toml_string()?;
        std::fs::write(path, body)
            .map_err(|e| ConfigError::Write(format!("write {}: {e}", path.display())))
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
    /// File could not be written.
    #[error("cannot write config: {0}")]
    Write(String),
}
