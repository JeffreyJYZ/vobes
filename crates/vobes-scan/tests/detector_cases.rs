//! Per-case detector tests. Each test pins a single framework/PM/language
//! signature so editing the signature table is loud when one breaks.
//! Order-sensitive: the JS framework table checks more-specific-before-less.

use std::fs;
use std::path::PathBuf;

use vobes_scan::{DefaultScanner, Scanner};

fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vobes-det-case-{name}-{}", std::process::id()));
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

fn detect(dir: &std::path::Path) -> vobes_scan::Detection {
    DefaultScanner::with_standard_detectors()
        .detect(dir)
        .unwrap()
}

// JS frameworks: order matters — Next.js must win over React, Remix over React,
// etc. Each test pins one signature.
#[test]
fn js_nextjs_over_react() {
    let dir = fixture("next");
    write(
        &dir,
        "package.json",
        r#"{"dependencies":{"next":"14","react":"18","react-dom":"18"}}"#,
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("Next.js"));
}

#[test]
fn js_remix_over_react() {
    let dir = fixture("remix");
    write(
        &dir,
        "package.json",
        r#"{"dependencies":{"@remix-run/react":"2","react":"18"}}"#,
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("Remix"));
}

#[test]
fn js_sveltekit_over_svelte() {
    let dir = fixture("sveltekit");
    write(
        &dir,
        "package.json",
        r#"{"dependencies":{"@sveltejs/kit":"2","svelte":"4"}}"#,
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("SvelteKit"));
}

#[test]
fn js_nuxt() {
    let dir = fixture("nuxt");
    write(&dir, "package.json", r#"{"dependencies":{"nuxt":"3"}}"#);
    assert_eq!(detect(&dir).framework.as_deref(), Some("Nuxt"));
}

#[test]
fn js_hono() {
    let dir = fixture("hono");
    write(&dir, "package.json", r#"{"dependencies":{"hono":"4"}}"#);
    assert_eq!(detect(&dir).framework.as_deref(), Some("Hono"));
}

#[test]
fn js_express() {
    let dir = fixture("express");
    write(&dir, "package.json", r#"{"dependencies":{"express":"4"}}"#);
    assert_eq!(detect(&dir).framework.as_deref(), Some("Express"));
}

#[test]
fn js_react() {
    let dir = fixture("react");
    write(
        &dir,
        "package.json",
        r#"{"dependencies":{"react":"18","react-dom":"18"}}"#,
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("React"));
}

// Rust frameworks — substring match on lowercase toml body.
#[test]
fn rust_axum() {
    let dir = fixture("axum");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname=\"x\"\n[dependencies]\naxum=\"0.7\"\n",
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("Axum"));
}

#[test]
fn rust_actix() {
    let dir = fixture("actix");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname=\"x\"\n[dependencies]\nactix-web=\"4\"\n",
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("Actix Web"));
}

#[test]
fn rust_tauri() {
    let dir = fixture("tauri-app");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname=\"x\"\n[dependencies]\ntauri=\"2\"\n",
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("Tauri"));
}

#[test]
fn rust_no_framework_falls_back() {
    let dir = fixture("plain-rust");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname=\"x\"\n[dependencies]\nserde=\"1\"\n",
    );
    assert_eq!(
        detect(&dir).framework.as_deref(),
        Some("Rust (no framework)")
    );
}

// Python frameworks.
#[test]
fn python_fastapi() {
    let dir = fixture("fastapi");
    write(
        &dir,
        "pyproject.toml",
        "[project]\nname=\"x\"\ndependencies=[\"fastapi\"]\n",
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("FastAPI"));
}

#[test]
fn python_django() {
    let dir = fixture("django");
    write(
        &dir,
        "pyproject.toml",
        "[project]\nname=\"x\"\ndependencies=[\"django\"]\n",
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("Django"));
}

#[test]
fn python_plain() {
    let dir = fixture("plain-py");
    write(&dir, "pyproject.toml", "[project]\nname=\"x\"\n");
    assert_eq!(detect(&dir).framework.as_deref(), Some("Python"));
}

// Go frameworks.
#[test]
fn go_gin() {
    let dir = fixture("gin");
    write(
        &dir,
        "go.mod",
        "module x\nrequire github.com/gin-gonic/gin v1.9\n",
    );
    assert_eq!(detect(&dir).framework.as_deref(), Some("Gin"));
}

#[test]
fn go_plain() {
    let dir = fixture("plain-go");
    write(&dir, "go.mod", "module x\n");
    assert_eq!(detect(&dir).framework.as_deref(), Some("Go (no framework)"));
}

// Package manager detector — first matching lockfile wins.
#[test]
fn pm_pnpm_wins_over_npm() {
    let dir = fixture("pnpm");
    write(&dir, "pnpm-lock.yaml", "lockfileVersion: 6.0\n");
    write(&dir, "package-lock.json", "{}");
    assert_eq!(detect(&dir).package_manager.as_deref(), Some("pnpm"));
}

#[test]
fn pm_uv_detected() {
    let dir = fixture("uv");
    write(&dir, "uv.lock", "version = 1\n");
    assert_eq!(detect(&dir).package_manager.as_deref(), Some("uv"));
}

// Repo detector — both .git dir and .git file (worktree) count.
#[test]
fn repo_worktree_via_dotgit_file() {
    let dir = fixture("worktree");
    fs::write(dir.join(".git"), "gitdir: /tmp/elsewhere\n").unwrap();
    let d = detect(&dir);
    assert!(d.is_repo);
}
