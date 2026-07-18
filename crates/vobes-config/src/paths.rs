//! Platform path resolution for Vobes files.
//!
//! `db_path()` and `snapshots_dir()` are consumed by the `vobes-store`
//! and CLI export commands in later phases. They are part of the public
//! API of this crate even when unused within the scaffold.

#![allow(dead_code)]

use std::path::PathBuf;

/// Directory that holds all vobes state (config, db, snapshots).
///
/// Follows platform conventions via the `dirs` crate:
/// - macOS: `~/Library/Application Support/vobes`
/// - Linux: `$XDG_CONFIG_HOME/vobes` (default `~/.config/vobes`)
/// - Windows: `%APPDATA%\vobes`
pub fn state_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("vobes"))
}

/// Path to the user config file.
pub fn config_path() -> Option<PathBuf> {
    state_dir().map(|d| d.join("config.toml"))
}

/// Path to the SQLite database file.
pub fn db_path() -> Option<PathBuf> {
    state_dir().map(|d| d.join("vobes.db"))
}

/// Path to the JSON snapshots directory.
pub fn snapshots_dir() -> Option<PathBuf> {
    state_dir().map(|d| d.join("snapshots"))
}
