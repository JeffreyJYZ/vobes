//! Repository detector — recognizes git projects.

use std::path::Path;
use vobes_core::Result;

use crate::detector::{Detection, Detector};

/// Detects git repositories by looking for `.git/` (directory),
/// `.git` (file → worktree), or submodule entries.
#[derive(Debug, Default, Clone, Copy)]
pub struct RepoDetector;

impl RepoDetector {
    /// Create a new repo detector.
    pub fn new() -> Self {
        Self
    }
}

impl Detector for RepoDetector {
    fn name(&self) -> &str {
        "repo"
    }

    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        if !path.is_dir() {
            return Ok(None);
        }
        let git = path.join(".git");
        // A `.git` directory, or a `.git` file pointing at one (worktree,
        // submodule). A stray empty `.git` file is not a repo — tools like
        // uv leave those inside cache dirs, and libgit2 then fails to open
        // it.
        let is_repo = git.is_dir()
            || std::fs::read_to_string(&git)
                .map(|s| s.trim_start().starts_with("gitdir:"))
                .unwrap_or(false);
        if is_repo {
            Ok(Some(Detection {
                is_repo: true,
                ..Default::default()
            }))
        } else {
            Ok(None)
        }
    }
}
