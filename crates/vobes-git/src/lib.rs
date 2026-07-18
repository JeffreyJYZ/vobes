//! Vobes Git module — read-only surfacing of git state.
//!
//! Does not commit, push, branch, or merge. Implementation arrives in
//! Phase 3.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use std::path::Path;
use vobes_core::{GitInfo, Result};

/// Read git state for the repo at `path`, if any.
///
/// Returns `Ok(None)` if the path is not a git repository.
pub fn read_git_info(_path: &Path) -> Result<Option<GitInfo>> {
    Ok(None)
}
