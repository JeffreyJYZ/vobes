//! Integration tests for the default scanner with fixture projects.

use std::fs;
use std::path::PathBuf;

use vobes_scan::{DefaultScanner, Scanner};

fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vobes-scan-test-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &std::path::Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, content).unwrap();
}

#[test]
fn detects_rust_project_with_cargo_and_framework() {
    let dir = fixture("rust");
    write(
        &dir,
        "Cargo.toml",
        "name = \"demo\"\nversion = \"0.1\"\n[dependencies]\ntauri = \"1\"\n",
    );
    write(&dir, "Cargo.lock", "");
    write(&dir, "src/main.rs", "fn main() {}\n");

    let scanner = DefaultScanner::with_standard_detectors().with_max_depth(2);
    let report = scanner.scan_report(&dir).unwrap();
    assert_eq!(report.count(), 1, "got {report:?}");

    let (_, detection) = &report.vobes[0];
    assert_eq!(detection.package_manager.as_deref(), Some("cargo"));
    assert_eq!(detection.framework.as_deref(), Some("Tauri"));
    assert_eq!(detection.language.as_deref(), Some("Rust"));
    assert!(!detection.is_repo);
}

#[test]
fn detects_node_project_with_pnpm() {
    let dir = fixture("node");
    write(&dir, "pnpm-lock.yaml", "lockfileVersion: 6.0\n");
    write(
        &dir,
        "package.json",
        r#"{"dependencies":{"next":"14","react":"18","react-dom":"18"}}"#,
    );
    write(
        &dir,
        "src/pages/index.tsx",
        "export default function H() { return null }\n",
    );

    let scanner = DefaultScanner::with_standard_detectors().with_max_depth(2);
    let report = scanner.scan_report(&dir).unwrap();
    assert_eq!(report.count(), 1);
    let (_, detection) = &report.vobes[0];
    assert_eq!(detection.package_manager.as_deref(), Some("pnpm"));
    assert_eq!(detection.framework.as_deref(), Some("Next.js"));
    assert_eq!(detection.language.as_deref(), Some("TypeScript"));
}

#[test]
fn detects_git_repo_via_dotgit_dir() {
    let dir = fixture("git");
    fs::create_dir_all(dir.join(".git")).unwrap();
    fs::write(dir.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    write(&dir, "README.md", "# demo\n");
    write(&dir, "main.py", "print(\"hi\")\n");

    let scanner = DefaultScanner::with_standard_detectors().with_max_depth(3);
    let report = scanner.scan_report(&dir).unwrap();
    assert_eq!(report.count(), 1);
    let (_, detection) = &report.vobes[0];
    assert!(detection.is_repo);
    assert_eq!(detection.language.as_deref(), Some("Python"));
}

#[test]
fn excludes_builtin_junk_dirs() {
    let dir = fixture("excluded");
    write(&dir, "Cargo.toml", "name = \"demo\"\n");
    write(&dir, "src/lib.rs", "pub fn x() {}\n");
    // node_modules under the project should be skipped
    write(&dir, "node_modules/foo.js", "module.exports = 1;\n");

    let scanner = DefaultScanner::with_standard_detectors().with_max_depth(3);
    let report = scanner.scan_report(&dir).unwrap();
    // Only one vobe — the project itself; node_modules excluded.
    assert_eq!(report.count(), 1);
    // Skipped counter incremented (>=1 for node_modules)
    assert!(
        report.dirs_skipped >= 1,
        "dirs_skipped = {}",
        report.dirs_skipped
    );
}

#[test]
fn respects_user_excludes() {
    let dir = fixture("user-excludes");
    write(&dir, "Cargo.toml", "name = \"demo\"\n");
    write(&dir, "src/lib.rs", "pub fn x() {}\n");
    // A sub-project we want to exclude
    write(&dir, "scratch/Cargo.toml", "name = \"scratch\"\n");
    write(&dir, "scratch/src/lib.rs", "pub fn y() {}\n");

    let scanner = DefaultScanner::with_standard_detectors()
        .with_max_depth(3)
        .with_extra_excludes(vec!["scratch".to_string()]);
    let report = scanner.scan_report(&dir).unwrap();
    assert_eq!(report.count(), 1);
    assert_eq!(report.vobes[0].0, dir);
}

#[test]
fn empty_dir_produces_no_vobes() {
    let dir = fixture("empty");
    let scanner = DefaultScanner::with_standard_detectors();
    let report = scanner.scan_report(&dir).unwrap();
    assert_eq!(report.count(), 0);
    assert!(report.is_empty());
}

#[test]
fn scan_via_trait_object() {
    let dir = fixture("trait");
    write(&dir, "go.mod", "module demo\n\ngo 1.21\n");
    write(&dir, "main.go", "package main\nfunc main() {}\n");
    write(&dir, "go.sum", "");

    let scanner: Box<dyn Scanner> = Box::new(DefaultScanner::with_standard_detectors());
    let vobes = scanner.scan(&dir).unwrap();
    assert_eq!(vobes.len(), 1);
    let (_, d) = &vobes[0];
    assert_eq!(d.package_manager.as_deref(), Some("go"));
    assert_eq!(d.language.as_deref(), Some("Go"));
}

#[test]
fn nextjs_prefers_typescript_over_css() {
    // A Next.js app: many .css files but the source is TypeScript/TSX.
    let dir = fixture("nextjs");
    write(
        &dir,
        "package.json",
        r#"{"dependencies":{"next":"14","react":"18"}}"#,
    );
    write(&dir, "tsconfig.json", "{}");
    write(&dir, "pages/index.tsx", "export default () => null;\n");
    write(&dir, "pages/about.tsx", "export default () => null;\n");
    // More CSS than TSX on purpose.
    for i in 0..10 {
        write(
            &dir,
            &format!("styles/{i}.module.css"),
            ".x { color: red; }\n",
        );
    }
    write(&dir, "global.css", "body { margin: 0; }\n");

    let scanner = DefaultScanner::with_standard_detectors().with_max_depth(4);
    let report = scanner.scan_report(&dir).unwrap();
    assert_eq!(report.count(), 1);
    let (_, detection) = &report.vobes[0];
    // Source language must beat styling/markup even when outnumbered.
    assert_eq!(detection.language.as_deref(), Some("TypeScript"));
}
