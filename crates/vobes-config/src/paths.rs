//! Platform path resolution for Vobes files.
//!
//! `db_path()` and `snapshots_dir()` are consumed by the `vobes-store`
//! and CLI export commands in later phases. They are part of the public
//! API of this crate even when unused within the scaffold.
//!
//! ## Dev vs release isolation
//!
//! Debug builds write to a `-dev`-suffixed directory so a `cargo tauri dev`
//! run can never trample data belonging to an installed release copy, and
//! vice versa. This is the same pattern Firefox, Zed, and others use.
//!
//! Override the dir name with the `VOBES_APP_DIR` env var if you need
//! a custom layout (e.g. for testing).

#![allow(dead_code)]

use std::path::PathBuf;

/// Suffix appended to the data directory in debug builds so dev and
/// release don't share a state dir.
#[cfg(debug_assertions)]
pub const APP_DIR_SUFFIX: &str = "-dev";
#[cfg(not(debug_assertions))]
pub const APP_DIR_SUFFIX: &str = "";

const APP_BASE: &str = "vobes";

/// Effective app data directory name. `"vobes"` for release,
/// `"vobes-dev"` for debug. Honors `VOBES_APP_DIR` if set.
pub fn app_dir_name() -> String {
    if let Ok(override_) = std::env::var("VOBES_APP_DIR") {
        if !override_.is_empty() {
            return override_;
        }
    }
    format!("{APP_BASE}{APP_DIR_SUFFIX}")
}

/// Directory that holds all vobes state (config, db, snapshots).
///
/// Follows platform conventions via the `dirs` crate:
/// - macOS: `~/Library/Application Support/<app_dir_name()>`
/// - Linux: `$XDG_CONFIG_HOME/<app_dir_name()>` (default `~/.config/<app_dir_name()>`)
/// - Windows: `%APPDATA%\<app_dir_name()>`
pub fn state_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(app_dir_name()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes tests that mutate `VOBES_APP_DIR`. `cargo test` runs
    /// test threads in parallel, and `std::env` mutation is not
    /// thread-safe, so the three env-reading tests must not interleave.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn app_dir_name_matches_profile() {
        let _guard = ENV_LOCK.lock().unwrap();
        let n = app_dir_name();
        if cfg!(debug_assertions) {
            assert!(
                n.ends_with("-dev"),
                "debug build should use -dev suffix, got {n}"
            );
        } else {
            assert!(
                !n.ends_with("-dev"),
                "release build should not use -dev suffix, got {n}"
            );
        }
    }

    #[test]
    fn state_dir_ends_with_app_name() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var("VOBES_APP_DIR").ok();
        std::env::remove_var("VOBES_APP_DIR");
        if let Some(d) = state_dir() {
            let name = d.file_name().and_then(|s| s.to_str()).unwrap_or("");
            assert_eq!(name, app_dir_name());
        }
        if let Some(v) = prev {
            std::env::set_var("VOBES_APP_DIR", v)
        }
    }

    #[test]
    fn env_override_takes_precedence() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var("VOBES_APP_DIR").ok();
        std::env::set_var("VOBES_APP_DIR", "vobes-test-override");
        assert_eq!(app_dir_name(), "vobes-test-override");
        match prev {
            Some(v) => std::env::set_var("VOBES_APP_DIR", v),
            None => std::env::remove_var("VOBES_APP_DIR"),
        }
    }
}
