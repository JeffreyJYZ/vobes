//! Tests for the scan trait surface and shared helpers.

use vobes_scan::{is_excluded, Detection, BUILTIN_EXCLUDES};

#[test]
fn builtin_excludes_block_known_junk() {
    assert!(is_excluded("node_modules", &[]));
    assert!(is_excluded(".git", &[]));
    assert!(is_excluded("target", &[]));
    assert!(is_excluded(".venv", &[]));
}

#[test]
fn user_excludes_extend_builtin() {
    let extras = vec!["scratch".to_string(), "tmp".to_string()];
    assert!(is_excluded("scratch", &extras));
    assert!(is_excluded("tmp", &extras));
    // Built-ins still apply
    assert!(is_excluded("dist", &extras));
}

#[test]
fn normal_dirs_are_not_excluded() {
    assert!(!is_excluded("my-app", &[]));
    // Source-only subdirs are not junk excludes (so language census still
    // sees their files); they are only excluded as vobe candidates.
    assert!(!is_excluded("src", &[]));
    assert!(!is_excluded("src-tauri", &[]));
}

#[test]
fn code_subdirs_excluded_as_candidates() {
    use vobes_scan::is_code_subdir;
    assert!(is_code_subdir("src"));
    assert!(is_code_subdir("src-tauri"));
    assert!(is_code_subdir("lib"));
    assert!(is_code_subdir("app"));
    assert!(!is_code_subdir("components"));
    assert!(!is_code_subdir("node_modules"));
}

#[test]
fn detection_empty_and_is_empty() {
    let d = Detection::empty();
    assert!(d.is_empty());
    // Builtin excludes list is non-empty by design
    assert!(!BUILTIN_EXCLUDES.is_empty());
}

#[test]
fn detection_merge_combines_fields() {
    let mut a = Detection {
        is_repo: true,
        framework: None,
        language: Some("Rust".into()),
        package_manager: None,
    };
    let b = Detection {
        is_repo: false,
        framework: Some("Tauri".into()),
        language: Some("TypeScript".into()),
        package_manager: Some("cargo".into()),
    };
    a.merge(b);
    // is_repo stays true (OR)
    assert!(a.is_repo);
    // framework filled from b
    assert_eq!(a.framework.as_deref(), Some("Tauri"));
    // language on a wins
    assert_eq!(a.language.as_deref(), Some("Rust"));
    // package_manager filled from b
    assert_eq!(a.package_manager.as_deref(), Some("cargo"));
}
