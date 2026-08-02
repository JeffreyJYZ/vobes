//! Cross-platform helpers for spawning a terminal or revealing a path
//! in the system file manager. Kept small and explicit per-OS.

use std::path::Path;
use std::process::Command;

/// Reveal a path in the OS file manager and select it if a file.
///
/// Errors are returned as `vobes_core::Error::internal` so the frontend
/// can surface them as a toast.
pub fn reveal(path: &Path) -> vobes_core::Result<()> {
    if !path.exists() {
        return Err(vobes_core::Error::not_found(path.display().to_string()));
    }
    let status = reveal_command(path)
        .status()
        .map_err(|e| vobes_core::Error::internal(format!("reveal: {e}")))?;
    if !status.success() {
        return Err(vobes_core::Error::internal(format!(
            "reveal exited with status {status}"
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn reveal_command(path: &Path) -> Command {
    let mut c = Command::new("open");
    if path.is_file() {
        c.arg("-R").arg(path);
    } else {
        c.arg(path);
    }
    c
}

#[cfg(target_os = "linux")]
fn reveal_command(path: &Path) -> Command {
    let mut c = Command::new("xdg-open");
    let dir = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    c.arg(dir);
    c
}

#[cfg(target_os = "windows")]
fn reveal_command(path: &Path) -> Command {
    let mut c = Command::new("explorer");
    if path.is_dir() {
        c.arg(path);
    } else {
        // explorer /select,path highlights the file.
        c.arg(format!("/select,{}", path.display()));
    }
    c
}

/// Spawn a new OS terminal at the given directory.
///
/// We deliberately pick portable commands instead of the user's
/// configured `$EDITOR`/`$SHELL` so this works on a fresh install
/// without configuration.
pub fn open_terminal(path: &Path) -> vobes_core::Result<()> {
    let dir = if path.is_dir() {
        path
    } else if let Some(parent) = path.parent() {
        parent
    } else {
        path
    };
    if !dir.exists() {
        return Err(vobes_core::Error::not_found(dir.display().to_string()));
    }
    let status = terminal_command(dir)
        .status()
        .map_err(|e| vobes_core::Error::internal(format!("terminal: {e}")))?;
    if !status.success() {
        return Err(vobes_core::Error::internal(format!(
            "terminal exited with status {status}"
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn terminal_command(dir: &Path) -> Command {
    let mut c = Command::new("open");
    c.arg("-a").arg("Terminal").arg(dir);
    c
}

#[cfg(target_os = "linux")]
fn terminal_command(dir: &Path) -> Command {
    // Best-effort across GNOME / KDE / XFCE / generic. We try x-terminal-emulator
    // first (Debian convention) then fall back to gnome-terminal, then xdg-open
    // which usually launches the user's default terminal handler.
    if which_exists("x-terminal-emulator") {
        let mut c = Command::new("x-terminal-emulator");
        c.current_dir(dir);
        c
    } else if which_exists("gnome-terminal") {
        let mut c = Command::new("gnome-terminal");
        c.current_dir(dir);
        c
    } else {
        let mut c = Command::new("xdg-open");
        c.arg(dir);
        c
    }
}

#[cfg(target_os = "windows")]
fn terminal_command(dir: &Path) -> Command {
    let mut c = Command::new("cmd");
    c.arg("/C")
        .arg("start")
        .arg("cmd")
        .arg("/K")
        .arg(format!("cd /D \"{}\"", dir.display()));
    c
}

#[cfg(target_os = "linux")]
fn which_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
