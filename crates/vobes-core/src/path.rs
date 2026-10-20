//! Path normalization helpers.
//!
//! Vobes stores project paths in SQLite. To keep lookups consistent
//! regardless of how a path is passed in (mixed separators, trailing
//! slashes, `.`/`..` segments), we normalize before storage and before
//! query.
//!
//! Rules:
//! - Convert `\` to `/` on all platforms (normalized storage form).
//! - Collapse consecutive separators.
//! - Strip trailing separators (except for root `/`).
//! - Resolve `.` segments (no-op) and `..` segments where possible.
//! - Do NOT touch leading `~` (that's config's job via `expand_home`).
//! - Do NOT canonicalize symlinks (too slow, may not exist yet).

use std::path::{Path, PathBuf};

/// Normalize a path to a canonical string form.
///
/// Returns a `PathBuf` whose `to_string_lossy()` is stable across
/// platforms and input separator styles.
pub fn normalize(path: &Path) -> PathBuf {
    let s = path.to_string_lossy().replace('\\', "/");
    let (drive, body) = match s.get(..2) {
        Some(d)
            if d.len() == 2 && d.as_bytes()[1] == b':' && d.as_bytes()[0].is_ascii_alphabetic() =>
        {
            (Some(d), &s[2..])
        }
        _ => (None, s.as_str()),
    };
    let absolute = body.starts_with('/');

    let mut parts: Vec<&str> = Vec::new();
    for seg in body.split('/') {
        match seg {
            "" | "." => {}
            ".." => match parts.last() {
                Some(prev) if *prev != ".." => {
                    parts.pop();
                }
                _ if !absolute && drive.is_none() => parts.push(".."),
                _ => {}
            },
            p => parts.push(p),
        }
    }

    let mut joined = String::new();
    if let Some(d) = drive {
        joined.push_str(d);
    }
    if absolute || drive.is_some() {
        joined.push('/');
    }
    joined.push_str(&parts.join("/"));
    if joined.is_empty() {
        joined.push('.');
    }
    PathBuf::from(joined)
}

/// Normalize a path-like string.
pub fn normalize_str(s: &str) -> PathBuf {
    normalize(Path::new(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_absolute_stays() {
        assert_eq!(
            normalize_str("/usr/local/bin"),
            PathBuf::from("/usr/local/bin")
        );
    }

    #[test]
    fn collapses_double_separators() {
        assert_eq!(normalize_str("/usr//local"), PathBuf::from("/usr/local"));
    }

    #[test]
    fn strips_trailing_slash() {
        assert_eq!(normalize_str("/usr/local/"), PathBuf::from("/usr/local"));
    }

    #[test]
    fn keeps_root() {
        assert_eq!(normalize_str("/"), PathBuf::from("/"));
    }

    #[test]
    fn resolves_dot_segments() {
        assert_eq!(normalize_str("/usr/./local"), PathBuf::from("/usr/local"));
    }

    #[test]
    fn resolves_dotdot() {
        assert_eq!(
            normalize_str("/usr/local/../bin"),
            PathBuf::from("/usr/bin")
        );
    }

    #[test]
    fn backslashes_normalized_to_forward() {
        assert_eq!(
            normalize_str("/usr\\local\\bin"),
            PathBuf::from("/usr/local/bin")
        );
    }

    #[test]
    fn windows_drive_preserved() {
        assert_eq!(normalize_str("C:\\dev\\foo"), PathBuf::from("C:/dev/foo"));
        assert_eq!(normalize_str("C:/dev/foo"), PathBuf::from("C:/dev/foo"));
    }

    #[test]
    fn windows_drive_dotdot() {
        assert_eq!(
            normalize_str("C:/dev/foo/../bar"),
            PathBuf::from("C:/dev/bar")
        );
    }

    #[test]
    fn relative_path_stays_relative() {
        assert_eq!(normalize_str("dev/foo"), PathBuf::from("dev/foo"));
    }

    #[test]
    fn relative_dotdot_kept() {
        assert_eq!(normalize_str("../foo"), PathBuf::from("../foo"));
        assert_eq!(normalize_str("../foo/.."), PathBuf::from(".."));
    }

    #[test]
    fn empty_becomes_dot() {
        assert_eq!(normalize_str(""), PathBuf::from("."));
    }
}
