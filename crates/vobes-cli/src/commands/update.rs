//! `vbs update` — point users at their package manager.
//!
//! `vbs` is published to crates.io. Detection tells you which command
//! to run on the current machine; we never overwrite the binary
//! in-place anymore (no sudo, no quarantine, no SHA256 dance — crates.io
//! handles signing and integrity).

use std::env;
use std::path::Path;

use vobes_core::Result;

const CARGO_CMD: &str = "cargo install vobes-cli --locked --force";

const MANAGED: &[(&str, &str, &str)] = &[
    (
        "/opt/homebrew/",
        "Homebrew (macOS)",
        "brew upgrade vobes-cli",
    ),
    (
        "/home/linuxbrew/.linuxbrew/",
        "Homebrew (Linux)",
        "brew upgrade vobes-cli",
    ),
    ("/snap/", "snap", "sudo snap refresh vobes-cli"),
];

pub fn run() -> Result<()> {
    let current = std::env::current_exe()
        .map_err(|e| vobes_core::Error::internal(format!("locate current binary: {e}")))?;
    let local = env!("CARGO_PKG_VERSION").to_string();
    println!("vbs {local}");

    let cargo_home = env::var("CARGO_HOME").ok();
    let home = env::var("HOME").ok();
    match detect_manager(&current, cargo_home.as_deref(), home.as_deref()) {
        Some((mgr, cmd)) => {
            println!("managed by {mgr}");
            println!("update with: {cmd}");
        }
        None => {
            println!("update with: {CARGO_CMD}");
        }
    }
    Ok(())
}

fn detect_manager(
    path: &Path,
    cargo_home: Option<&str>,
    home: Option<&str>,
) -> Option<(&'static str, &'static str)> {
    let s = path.to_string_lossy();
    if let Some(cargo_home) = cargo_home {
        if s.starts_with(&format!("{cargo_home}/bin/")) {
            return Some(("cargo", CARGO_CMD));
        }
    }
    if let Some(home) = home {
        if s.starts_with(&format!("{home}/.cargo/bin/")) {
            return Some(("cargo", CARGO_CMD));
        }
    }
    MANAGED
        .iter()
        .find(|(prefix, _, _)| s.starts_with(prefix))
        .map(|(_, mgr, cmd)| (*mgr, *cmd))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_home_match() {
        assert_eq!(
            detect_manager(Path::new("/x/cargo/bin/vbs"), Some("/x/cargo"), None).unwrap(),
            ("cargo", CARGO_CMD)
        );
    }

    #[test]
    fn home_cargo_match() {
        assert_eq!(
            detect_manager(Path::new("/x/.cargo/bin/vbs"), None, Some("/x")).unwrap(),
            ("cargo", CARGO_CMD)
        );
    }

    #[test]
    fn homebrew_match() {
        assert_eq!(
            detect_manager(Path::new("/opt/homebrew/bin/vbs"), None, None)
                .unwrap()
                .0,
            "Homebrew (macOS)"
        );
    }

    #[test]
    fn unmanaged_returns_none() {
        assert!(detect_manager(Path::new("/x/.local/bin/vbs"), None, Some("/x")).is_none());
    }
}
