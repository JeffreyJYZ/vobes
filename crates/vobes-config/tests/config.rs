//! Tests for config loading, defaults, and path helpers.

use std::fs;
use vobes_config::{expand_home, Config};

#[test]
fn empty_toml_uses_defaults() {
    let c = Config::from_toml_str("").unwrap();
    assert_eq!(c.scan.max_depth, 4);
    assert!(!c.scan.follow_symlinks);
    assert_eq!(c.display.theme, "auto");
}

#[test]
fn partial_toml_overrides_only_set_fields() {
    let toml = r#"
[scan]
max_depth = 8
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(c.scan.max_depth, 8);
    assert!(!c.scan.follow_symlinks);
    assert_eq!(c.display.theme, "auto");
}

#[test]
fn unknown_field_is_ignored() {
    // A stale or mistyped key must not drop the rest of the config —
    // one unknown field used to make every setting revert to defaults.
    let toml = r#"
[scan]
unknown_field = true
max_depth = 8
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(c.scan.max_depth, 8);
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
[scan]
max_depth = 2
"#,
    )
    .unwrap();
    let c = Config::load_from(&p).unwrap();
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
    assert!(!roots[0].to_string_lossy().contains('~'));
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

#[test]
fn desktop_section_defaults_to_off() {
    let c = Config::from_toml_str("").unwrap();
    assert!(!c.desktop.notify_behind);
}

#[test]
fn desktop_section_parses_when_present() {
    let toml = r#"
[desktop]
notify_behind = true
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert!(c.desktop.notify_behind);
}

#[test]
fn unknown_section_is_ignored() {
    // Sections dropped by the audit ([general], [git], [export]) must
    // parse harmlessly instead of failing the whole file.
    let toml = r#"
[export]
format = "json"

[scan]
max_depth = 6
"#;
    let c = Config::from_toml_str(toml).unwrap();
    assert_eq!(c.scan.max_depth, 6);
}

#[test]
fn save_to_round_trips() {
    use vobes_config::Config;
    let dir = std::env::temp_dir().join("vobes-config-save-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("config.toml");
    let mut c = Config::default();
    c.scan.max_depth = 7;
    c.desktop.notify_behind = true;
    c.save_to(&p).unwrap();
    let loaded = Config::load_from(&p).unwrap();
    assert_eq!(loaded.scan.max_depth, 7);
    assert!(loaded.desktop.notify_behind);
    std::fs::remove_file(&p).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn pre_audit_config_loads_and_self_heals() {
    // Pre-audit configs carried [general], [git], [export] sections and
    // stale fields inside surviving sections (display.date_format,
    // desktop.launch_on_login). Both must load without losing the rest —
    // and the file should be rewritten in canonical shape afterwards.
    let dir = std::env::temp_dir().join("vobes-config-migrate-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("config.toml");
    let legacy = r#"
[general]
name = "Personal Workspace"

[scan]
roots = ["~/dev", "~/work"]
exclude = ["scratch", "experiments"]
max_depth = 4
follow_symlinks = false

[display]
theme = "dark"
date_format = "relative"
default_sort = "last_modified"

[git]
cache_ttl_seconds = 60
fetch_upstream = false

[export]
format = "json"

[desktop]
notify_behind = true
launch_on_login = false
"#;
    std::fs::write(&p, legacy).unwrap();
    let loaded = Config::load_from(&p).unwrap();
    assert_eq!(loaded.scan.max_depth, 4);
    assert!(!loaded.scan.follow_symlinks);
    assert_eq!(
        loaded.scan.roots,
        vec!["~/dev".to_string(), "~/work".to_string()]
    );
    assert_eq!(
        loaded.scan.exclude,
        vec!["scratch".to_string(), "experiments".to_string()]
    );
    assert_eq!(loaded.display.theme, "dark");
    assert!(loaded.desktop.notify_behind);

    let rewritten = std::fs::read_to_string(&p).unwrap();
    assert!(!rewritten.contains("[git]"), "stale section survived");
    assert!(
        !rewritten.contains("launch_on_login"),
        "stale field survived"
    );
    std::fs::remove_file(&p).ok();
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn shipped_example_toml_parses() {
    // The vobes.example.toml at the repo root must always parse
    // against the current schema — catches stale fields after a
    // refactor.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let example = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("vobes.example.toml");
    if !example.exists() {
        // Workspace layout may differ in some CI setups; skip silently.
        return;
    }
    let s = std::fs::read_to_string(&example).unwrap();
    Config::from_toml_str(&s).expect("vobes.example.toml must parse");
}
