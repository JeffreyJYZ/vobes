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

/// Spawn a terminal at `path`. If `app` is `None` (or refers to a
/// terminal we do not know about), we fall back to the platform
/// default. On macOS `app` is the app name as passed to `open -a`
/// (e.g. `"iTerm"`, `"Terminal"`); on Linux it is an executable name
/// resolvable on `$PATH`; on Windows it is one of `"wt"`,
/// `"powershell"`, `"pwsh"`, or `"cmd"`.
pub fn open_terminal_with(path: &Path, app: Option<&str>) -> vobes_core::Result<()> {
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
    let mut cmd = terminal_command(dir, app);
    let status = cmd
        .status()
        .map_err(|e| vobes_core::Error::internal(format!("terminal: {e}")))?;
    if !status.success() {
        return Err(vobes_core::Error::internal(format!(
            "terminal exited with status {status}"
        )));
    }
    Ok(())
}

/// A terminal the user could choose to spawn.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TerminalApp {
    /// Stable id / key passed back into `open_terminal_with`.
    pub id: String,
    /// Human label shown in the dropdown.
    pub label: String,
    /// True when `id` is the platform default (highlighted first).
    pub is_default: bool,
}

/// List terminal emulators installed on this machine. Backs the
/// "Open in terminal" selector on the Projects detail view.
pub fn list_terminals() -> Vec<TerminalApp> {
    list_terminals_platform()
}

/// An editor the user could choose to open a vobe in. Some editors
/// are GUI apps (`open -a` on macOS) and some are CLI commands
/// (`$EDITOR`-style). The frontend does not care which — it just
/// passes `id` back into `open_in_editor`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EditorApp {
    /// Stable id / key passed to `open_in_editor`.
    pub id: String,
    /// Human label shown in the dropdown.
    pub label: String,
    /// True when `id` is the platform default.
    pub is_default: bool,
}

/// List editors installed on this machine. Backs the in-app
/// editor selector on the Projects detail view.
pub fn list_editors() -> Vec<EditorApp> {
    list_editors_platform()
}

/// Open a vobe directory in the given editor. `app` is the id from
/// `list_editors`. When `None` (or unknown), fall back to the
/// platform default, then `$EDITOR`, then a sensible hard-coded
/// choice (VS Code on macOS/Linux, `notepad` on Windows).
pub fn open_editor(path: &Path, app: Option<&str>) -> vobes_core::Result<()> {
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
    let mut cmd = editor_command(dir, app);
    let status = cmd
        .status()
        .map_err(|e| vobes_core::Error::internal(format!("editor: {e}")))?;
    if !status.success() {
        return Err(vobes_core::Error::internal(format!(
            "editor exited with status {status}"
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn terminal_command(dir: &Path, app: Option<&str>) -> Command {
    let app = app.unwrap_or("Terminal");
    let mut c = Command::new("open");
    c.arg("-a").arg(app).arg(dir);
    c
}

#[cfg(target_os = "macos")]
fn list_terminals_platform() -> Vec<TerminalApp> {
    // Candidate names (the value passed to `open -a`). Each is the
    // app's bundle name without the `.app` suffix. We probe the
    // canonical install locations and report whichever exists.
    let candidates = [
        ("Terminal", "/System/Applications/Utilities/Terminal.app"),
        ("iTerm", "/Applications/iTerm.app"),
        ("Warp", "/Applications/Warp.app"),
        ("Alacritty", "/Applications/Alacritty.app"),
        ("kitty", "/Applications/kitty.app"),
        ("WezTerm", "/Applications/WezTerm.app"),
        ("Hyper", "/Applications/Hyper.app"),
        ("Tabby", "/Applications/Tabby.app"),
        ("Wave", "/Applications/Wave.app"),
        ("Ghostty", "/Applications/Ghostty.app"),
    ];
    let mut out = Vec::new();
    for (id, path) in candidates {
        if std::path::Path::new(path).exists() {
            out.push(TerminalApp {
                id: id.to_string(),
                label: id.to_string(),
                is_default: id == "Terminal",
            });
        }
    }
    // Always include the default even if the probe missed it.
    if !out.iter().any(|t| t.is_default) {
        out.insert(
            0,
            TerminalApp {
                id: "Terminal".into(),
                label: "Terminal".into(),
                is_default: true,
            },
        );
    }
    out
}

#[cfg(target_os = "macos")]
fn editor_command(dir: &Path, app: Option<&str>) -> Command {
    let app = app.unwrap_or("Visual Studio Code");
    // CLI shims (`code`, `cursor`, `zed`, `subl`, `xed`, `bbedit`)
    // are faster and arg-correct on macOS — prefer them when they
    // resolve on $PATH, otherwise fall through to `open -a <App>`.
    let shim = match app {
        "Visual Studio Code" => Some("code"),
        "Cursor" => Some("cursor"),
        "Zed" => Some("zed"),
        "Sublime Text" => Some("subl"),
        "BBEdit" => Some("bbedit"),
        "Xcode" => Some("xed"),
        "Nova" => Some("nova"),
        _ => None,
    };
    if let Some(sh) = shim {
        if which_exists(sh) {
            let mut c = Command::new(sh);
            c.arg(dir);
            return c;
        }
    }
    let mut c = Command::new("open");
    c.arg("-a").arg(app).arg(dir);
    c
}

#[cfg(target_os = "macos")]
fn list_editors_platform() -> Vec<EditorApp> {
    let candidates = [
        ("Visual Studio Code", "/Applications/Visual Studio Code.app"),
        ("Cursor", "/Applications/Cursor.app"),
        ("Zed", "/Applications/Zed.app"),
        ("Sublime Text", "/Applications/Sublime Text.app"),
        ("BBEdit", "/Applications/BBEdit.app"),
        ("Nova", "/Applications/Nova.app"),
        ("Xcode", "/Applications/Xcode.app"),
        ("TextEdit", "/System/Applications/TextEdit.app"),
    ];
    let mut out = Vec::new();
    for (id, path) in candidates {
        if std::path::Path::new(path).exists() {
            out.push(EditorApp {
                id: id.to_string(),
                label: id.to_string(),
                is_default: id == "Visual Studio Code",
            });
        }
    }
    // Respect $EDITOR when it points at a CLI editor — prepend it
    // as a "Custom ($EDITOR)" entry so the user sees a real label.
    if let Ok(ed) = std::env::var("EDITOR") {
        if !ed.is_empty() {
            out.insert(
                0,
                EditorApp {
                    id: format!("$EDITOR:{ed}"),
                    label: format!("$EDITOR ({ed})"),
                    is_default: true,
                },
            );
            // downgrade the default flag on the others
            for e in out.iter_mut().skip(1) {
                e.is_default = false;
            }
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn terminal_command(dir: &Path, app: Option<&str>) -> Command {
    let app = app
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_linux_terminal());
    let mut c = Command::new(app);
    c.current_dir(dir);
    // gnome-terminal needs --working Directory to land on the dir.
    if app == "gnome-terminal" {
        c.arg(format!("--working-directory={}", dir.display()));
    }
    c
}

#[cfg(target_os = "linux")]
fn editor_command(dir: &Path, app: Option<&str>) -> Command {
    let app = app
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_linux_editor());
    let mut c = Command::new(app);
    c.arg(dir);
    c
}

#[cfg(target_os = "linux")]
fn default_linux_editor() -> String {
    std::env::var("EDITOR")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "code".into())
}

#[cfg(target_os = "linux")]
fn list_editors_platform() -> Vec<EditorApp> {
    let candidates = [
        ("code", "Visual Studio Code"),
        ("code-insiders", "VS Code Insiders"),
        ("cursor", "Cursor"),
        ("zed", "Zed"),
        ("subl", "Sublime Text"),
        ("vim", "Vim"),
        ("nvim", "Neovim"),
        ("emacs", "Emacs"),
        ("kak", "Kakoune"),
        ("micro", "micro"),
    ];
    let mut out: Vec<EditorApp> = candidates
        .iter()
        .filter(|(id, _)| which_exists(id))
        .map(|(id, label)| EditorApp {
            id: id.to_string(),
            label: label.to_string(),
            is_default: false,
        })
        .collect();
    if let Ok(ed) = std::env::var("EDITOR") {
        if !ed.is_empty() {
            out.insert(
                0,
                EditorApp {
                    id: format!("$EDITOR:{ed}"),
                    label: format!("$EDITOR ({ed})"),
                    is_default: true,
                },
            );
            for e in out.iter_mut().skip(1) {
                e.is_default = false;
            }
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn default_linux_terminal() -> &'static str {
    if which_exists("x-terminal-emulator") {
        "x-terminal-emulator"
    } else if which_exists("gnome-terminal") {
        "gnome-terminal"
    } else if which_exists("konsole") {
        "konsole"
    } else if which_exists("xfce4-terminal") {
        "xfce4-terminal"
    } else {
        "xterm"
    }
}

#[cfg(target_os = "linux")]
fn list_terminals_platform() -> Vec<TerminalApp> {
    let candidates = [
        "x-terminal-emulator",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "alacritty",
        "kitty",
        "wezterm",
        "foot",
        "xterm",
    ];
    let default = default_linux_terminal();
    let mut out = Vec::new();
    for id in candidates {
        if which_exists(id) {
            out.push(TerminalApp {
                id: id.to_string(),
                label: id.to_string(),
                is_default: id == default,
            });
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn terminal_command(dir: &Path, app: Option<&str>) -> Command {
    let app = app.unwrap_or("cmd");
    let mut c = Command::new("cmd");
    c.arg("/C")
        .arg("start")
        .arg(app)
        .arg("/K")
        .arg(format!("cd /D \"{}\"", dir.display()));
    c
}

#[cfg(target_os = "windows")]
fn editor_command(dir: &Path, app: Option<&str>) -> Command {
    let app = app.unwrap_or("code");
    let mut c = Command::new(app);
    c.arg(dir);
    c
}

#[cfg(target_os = "windows")]
fn list_editors_platform() -> Vec<EditorApp> {
    let candidates = [
        ("code", "Visual Studio Code"),
        ("code-insiders", "VS Code Insiders"),
        ("cursor", "Cursor"),
        ("zed", "Zed"),
        ("notepad", "Notepad"),
    ];
    let mut out: Vec<EditorApp> = candidates
        .iter()
        .filter(|(id, _)| which_exists(id))
        .map(|(id, label)| EditorApp {
            id: id.to_string(),
            label: label.to_string(),
            is_default: *id == "code",
        })
        .collect();
    if out.is_empty() {
        out.push(EditorApp {
            id: "notepad".into(),
            label: "Notepad".into(),
            is_default: true,
        });
    }
    out
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
fn which_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn list_terminals_platform() -> Vec<TerminalApp> {
    // We cannot reliably probe the Store-installed Windows Terminal
    // without a COM round-trip; list both, mark cmd as default.
    let candidates = [
        ("wt", "Windows Terminal"),
        ("pwsh", "PowerShell 7"),
        ("powershell", "Windows PowerShell"),
        ("cmd", "Command Prompt"),
    ];
    candidates
        .iter()
        .map(|(id, label)| TerminalApp {
            id: id.to_string(),
            label: label.to_string(),
            is_default: *id == "cmd",
        })
        .collect()
}
