//! `vbs update` — self-update the CLI binary in place.
//!
//! Downloads the matching asset from the latest GitHub release, verifies
//! its SHA256 against a sidecar `.sha256` asset, and atomically replaces
//! the running binary.
//!
//! No `sudo`, no macOS quarantine removal — we assume `vbs` lives in a
//! user-writable directory (e.g. `~/.cargo/bin`, `~/.local/bin`,
//! Homebrew on Apple Silicon). If it doesn't, the rename fails with a
//! clear message; reinstalling via your package manager is the fix.

use std::env;
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use vobes_core::Result;

const REPO_OWNER: &str = "JeffreyJYZ";
const REPO_NAME: &str = "vobes";
const API_BASE: &str = "https://api.github.com";
const ASSET_HOSTS: &[&str] = &["github.com", "objects.githubusercontent.com"];
const MAX_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn run(check_only: bool, target_override: Option<&str>, insecure: bool) -> Result<()> {
    let current = std::env::current_exe()
        .map_err(|e| vobes_core::Error::internal(format!("locate current binary: {e}")))?;
    let current = current.canonicalize().unwrap_or(current);
    let asset_name = asset_name_for(target_override)?;
    let local = env!("CARGO_PKG_VERSION").to_string();

    let release = fetch_latest_release()?;
    let tag = release.tag_name.trim_start_matches('v').to_string();
    if tag == local {
        println!("vbs {tag} (latest)");
        return Ok(());
    }
    println!("update available: {local} → {tag}");

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            vobes_core::Error::internal(format!("no asset {asset_name} in release {tag}"))
        })?;
    let sha = release
        .assets
        .iter()
        .find(|a| a.name == format!("{asset_name}.sha256"));

    if check_only {
        return Ok(());
    }

    let url = check_https_asset_url(&asset.browser_download_url)?;
    println!("fetching {url}");
    let bytes = download(&url)?;

    let expected_hex = if let Some(meta) = sha {
        let url = check_https_asset_url(&meta.browser_download_url)?;
        let body = download(&url)?;
        let s = std::str::from_utf8(&body)
            .map_err(|e| vobes_core::Error::internal(format!("sha256 utf8: {e}")))?;
        s.split_whitespace()
            .next()
            .map(|s| s.to_ascii_lowercase())
            .ok_or_else(|| vobes_core::Error::internal("empty sha256 file"))
    } else if insecure {
        Ok(String::new())
    } else {
        Err(vobes_core::Error::internal(format!(
            "no {asset_name}.sha256 sidecar in release; pass --insecure to skip verification"
        )))
    }?;

    if !expected_hex.is_empty() {
        let actual = hex_lower(Sha256::digest(&bytes));
        if actual != expected_hex {
            return Err(vobes_core::Error::internal(format!(
                "checksum mismatch: expected {expected_hex} got {actual}"
            )));
        }
    }

    let tmp = tmp_sibling(&current)?;
    fs::write(&tmp, &bytes)
        .map_err(|e| vobes_core::Error::internal(format!("write temp {}: {e}", tmp.display())))?;
    #[cfg(unix)]
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755)).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        vobes_core::Error::internal(format!("chmod 0755 {}: {e}", tmp.display()))
    })?;

    if let Err(e) = fs::rename(&tmp, &current) {
        let _ = fs::remove_file(&tmp);
        return Err(vobes_core::Error::internal(format!(
            "replace {}: {e}\nreinstall via your package manager (cargo install --path ., brew, …) so vbs lives in a writable bin dir",
            current.display()
        )));
    }

    println!("updated {local} → {tag} at {}", current.display());
    Ok(())
}

fn fetch_latest_release() -> Result<Release> {
    let url = format!("{API_BASE}/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");
    let resp = ureq::get(&url)
        .set("User-Agent", "vbs-update")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| vobes_core::Error::internal(format!("github api: {e}")))?;
    let reader = resp.into_reader();
    serde_json::from_reader(reader)
        .map_err(|e| vobes_core::Error::internal(format!("parse release: {e}")))
}

fn download(url: &str) -> Result<Vec<u8>> {
    let resp = ureq::get(url)
        .set("User-Agent", "vbs-update")
        .call()
        .map_err(|e| vobes_core::Error::internal(format!("download {url}: {e}")))?;
    if let Some(len) = resp.header("Content-Length") {
        if let Ok(n) = len.parse::<u64>() {
            if n > MAX_BYTES {
                return Err(vobes_core::Error::internal(format!(
                    "asset too large: {n} bytes (max {MAX_BYTES})"
                )));
            }
        }
    }
    let mut buf = Vec::new();
    let mut body = resp.into_reader();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let n = body
            .read(&mut chunk)
            .map_err(|e| vobes_core::Error::internal(format!("read body: {e}")))?;
        if n == 0 {
            break;
        }
        if buf.len() as u64 + n as u64 > MAX_BYTES {
            return Err(vobes_core::Error::internal(format!(
                "asset exceeds {MAX_BYTES} bytes mid-stream"
            )));
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    Ok(buf)
}

fn asset_name_for(override_name: Option<&str>) -> Result<String> {
    if let Some(name) = override_name {
        return Ok(name.to_string());
    }
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let stem = match (os, arch) {
        ("macos", "aarch64") => "vbs-macos-aarch64",
        ("macos", "x86_64") => "vbs-macos-x64",
        ("linux", "x86_64") => "vbs-linux-x64",
        ("linux", "aarch64") => "vbs-linux-aarch64",
        ("windows", "x86_64") => "vbs-windows-x64.exe",
        _ => {
            return Err(vobes_core::Error::internal(format!(
                "no published asset for {os}/{arch}"
            )))
        }
    };
    Ok(stem.to_string())
}

fn tmp_sibling(target: &std::path::Path) -> Result<PathBuf> {
    let pid = std::process::id();
    let mut name = target
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(format!(".update.{pid}"));
    Ok(target.with_file_name(name))
}

fn check_https_asset_url(url: &str) -> Result<String> {
    if !url.starts_with("https://") {
        return Err(vobes_core::Error::internal(format!(
            "refusing non-https url: {url}"
        )));
    }
    let host = url
        .strip_prefix("https://")
        .and_then(|s| s.split('/').next())
        .unwrap_or("");
    if !ASSET_HOSTS.contains(&host) {
        return Err(vobes_core::Error::internal(format!(
            "refusing asset url with unexpected host: {host}"
        )));
    }
    Ok(url.to_string())
}

fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    let mut out = String::with_capacity(bytes.as_ref().len() * 2);
    for b in bytes.as_ref() {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_name_override_passes_through() {
        assert_eq!(asset_name_for(Some("custom-name")).unwrap(), "custom-name");
    }

    #[test]
    fn asset_name_current_target_is_known() {
        // Whatever triple we compiled on, the resolver should produce
        // one of our published asset names — otherwise update can
        // never find a download.
        let name = asset_name_for(None).unwrap();
        assert!(
            [
                "vbs-macos-aarch64",
                "vbs-macos-x64",
                "vbs-linux-x64",
                "vbs-linux-aarch64",
                "vbs-windows-x64.exe",
            ]
            .contains(&name.as_str()),
            "unexpected asset name: {name}"
        );
    }

    #[test]
    fn rejects_non_https() {
        assert!(check_https_asset_url("http://github.com/x").is_err());
        assert!(check_https_asset_url("https://evil.example.com/x").is_err());
        assert!(check_https_asset_url("https://github.com/x").is_ok());
        assert!(check_https_asset_url("https://objects.githubusercontent.com/x").is_ok());
    }

    #[test]
    fn sha256_matches() {
        let actual = hex_lower(Sha256::digest(b"hello"));
        assert_eq!(
            actual,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
