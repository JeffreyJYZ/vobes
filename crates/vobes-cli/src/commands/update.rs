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

pub fn run(_check: bool, _target: Option<&str>, _insecure: bool) -> Result<()> {
    let current = std::env::current_exe()
        .map_err(|e| vobes_core::Error::internal(format!("locate current binary: {e}")))?;
    let local = env!("CARGO_PKG_VERSION").to_string();
    println!("vbs {local}");

    match detect_manager(&current) {
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

fn detect_manager(path: &Path) -> Option<(&'static str, &'static str)> {
    let s = path.to_string_lossy();
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        if s.starts_with(&format!("{cargo_home}/bin/")) {
            return Some(("cargo", CARGO_CMD));
        }
    }
    if let Ok(home) = env::var("HOME") {
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
        env::set_var("CARGO_HOME", "/x/cargo");
        env::remove_var("HOME");
        assert_eq!(
            detect_manager(Path::new("/x/cargo/bin/vbs")).unwrap(),
            ("cargo", CARGO_CMD)
        );
    }

    #[test]
    fn home_cargo_match() {
        env::remove_var("CARGO_HOME");
        env::set_var("HOME", "/x");
        assert_eq!(
            detect_manager(Path::new("/x/.cargo/bin/vbs")).unwrap(),
            ("cargo", CARGO_CMD)
        );
    }

    #[test]
    fn homebrew_match() {
        env::remove_var("CARGO_HOME");
        env::set_var("HOME", "/x");
        assert_eq!(
            detect_manager(Path::new("/opt/homebrew/bin/vbs"))
                .unwrap()
                .0,
            "Homebrew (macOS)"
        );
    }

    #[test]
    fn unmanaged_returns_none() {
        env::remove_var("CARGO_HOME");
        env::set_var("HOME", "/x");
        assert!(detect_manager(Path::new("/x/.local/bin/vbs")).is_none());
    }
}
