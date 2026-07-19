//! Exclude rules for the scanner.

/// Junk/build directories always excluded from every scan descent.
///
/// These are never projects and contain no useful source we want to
/// attribute to a parent project. Used both for candidate discovery and
/// for the language census.
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

/// Source-only subdirs that live *inside* a real project but are not
/// projects themselves. Excluded from candidate discovery (so `src`,
/// `src-tauri`, etc. are not mistaken for standalone vobes) but NOT from
/// the language census — their files still belong to the parent project.
pub const CODE_SUBDIR_EXCLUDES: &[&str] = &["src", "src-tauri", "lib", "app"];

/// Whether a directory name should be excluded from descent.
///
/// Matches against the built-in junk excludes plus user-supplied extras.
/// `name` is the file name component (not a full path).
pub fn is_excluded(name: &str, extra: &[String]) -> bool {
    if BUILTIN_EXCLUDES.contains(&name) {
        return true;
    }
    extra.iter().any(|e| e == name)
}

/// Whether a directory is a source-only subdir that should never be
/// treated as its own vobe candidate.
pub fn is_code_subdir(name: &str) -> bool {
    CODE_SUBDIR_EXCLUDES.contains(&name)
}
