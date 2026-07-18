//! Exclude rules for the scanner.

/// Built-in directories always excluded from scan descent.
///
/// User config may add more via `scan.exclude`. These cannot be
/// overridden — they are always excluded to keep scans fast and safe.
pub const BUILTIN_EXCLUDES: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    ".cache",
    "vendor",
    ".next",
    ".venv",
    ".idea",
    ".vscode",
    "__pycache__",
    ".mypy_cache",
    ".pytest_cache",
    ".gradle",
    ".sass-cache",
    "*.egg-info",
];

/// Whether a directory name should be excluded from descent.
///
/// Matches against the built-in excludes plus user-supplied extras.
/// `name` is the file name component (not a full path).
pub fn is_excluded(name: &str, extra: &[String]) -> bool {
    if BUILTIN_EXCLUDES.contains(&name) {
        return true;
    }
    extra.iter().any(|e| e == name)
}
