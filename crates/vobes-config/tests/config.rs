//! Tests for config loading, defaults, and path helpers.

use std::fs;
use vobes_config::{expand_home, Config};

#[test]
fn empty_toml_uses_defaults() {
    let c = Config::from_toml_str("").unwrap();
    assert_eq!(c.scan.max_depth, 4);
    assert!(!c.scan.follow_symlinks);
    assert_eq!(c.display.theme, "auto");
    assert_eq!(c.display.date_format, "relative");
    assert_eq!(c.display.default_sort, "last_modified");
    assert_eq!(c.git.cache_ttl_seconds, 60);
    assert!(!c.git.fetch_upstream);
    assert_eq!(c.export.format, "json");
}

#[test]
fn partial_toml_overrides_only_set_fields() {
    let toml = r#"
[scan]
max_depth = 8
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(c.scan.max_depth, 8);
    // Other defaults remain
    assert!(!c.scan.follow_symlinks);
    assert_eq!(c.display.theme, "auto");
}

#[test]
fn unknown_field_is_rejected() {
    let toml = r#"
[scan]
unknown_field = true
"#;
    assert!(Config::from_toml_str(toml).is_err());
}

#[test]
fn load_from_missing_path_returns_default() {
    let p = std::env::temp_dir().join("vobes-does-not-exist.toml");
    let c = Config::load_from(&p).unwrap();
    assert_eq!(c.scan.max_depth, 4);
}

#[test]
fn load_from_existing_path_parses() {
    let dir = std::env::temp_dir().join("vobes-config-tests");
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("config.toml");
    fs::write(
        &p,
        r#"
[general]
name = "Test"

[scan]
max_depth = 2
"#,
    )
    .unwrap();
    let c = Config::load_from(&p).unwrap();
    assert_eq!(c.general.name.as_deref(), Some("Test"));
    assert_eq!(c.scan.max_depth, 2);
    fs::remove_file(&p).ok();
    fs::remove_dir(&dir).ok();
}

#[test]
fn resolved_roots_expands_home() {
    let toml = r#"
[scan]
roots = ["~/dev", "/abs/path"]
"#;
    let c = Config::from_toml_str(toml).unwrap();
    let roots = c.resolved_roots();
    assert_eq!(roots.len(), 2);
    // First root expanded past `~`
    assert!(!roots[0].to_string_lossy().contains('~'));
    // Absolute path kept as-is
    assert_eq!(roots[1].to_string_lossy(), "/abs/path");
}

#[test]
fn expand_home_handles_corner_cases() {
    assert_eq!(
        expand_home("/abs").unwrap(),
        std::path::PathBuf::from("/abs")
    );
    let home = expand_home("~").unwrap();
    assert!(!home.to_string_lossy().contains('~'));
    let sub = expand_home("~/dev").unwrap();
    assert!(sub.to_string_lossy().ends_with("dev"));
    assert!(!sub.to_string_lossy().contains('~'));
}
