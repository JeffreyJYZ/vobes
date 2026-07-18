//! Vobes Git module — read-only surfacing of git state.
//!
//! Does not commit, push, branch, or merge.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use chrono::{DateTime, Utc};
use std::path::Path;
use vobes_core::{Commit, GitInfo, Result};

/// Read git state for the repo at `path`, if any.
///
/// Returns `Ok(None)` if the path is not a git repository.
///
/// All operations are read-only. We never call `git fetch` or any
/// mutating command.
pub fn read_git_info(path: &Path) -> Result<Option<GitInfo>> {
    let repo = match git2::Repository::discover(path) {
        Ok(r) => r,
        Err(e) if e.code() == git2::ErrorCode::NotFound => return Ok(None),
        Err(e) => return Err(vobes_core::Error::git(e.to_string())),
    };

    let branch = current_branch(&repo).unwrap_or_else(|| "(detached)".to_string());
    let dirty = is_dirty(&repo).unwrap_or(false);
    let (ahead, behind) = ahead_behind(&repo).unwrap_or((0, 0));
    let last_commit = last_commit(&repo).unwrap_or(None);

    Ok(Some(GitInfo {
        branch,
        dirty,
        ahead,
        behind,
        last_commit,
    }))
}

fn current_branch(repo: &git2::Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if head.is_branch() {
        return head.shorthand().map(str::to_string);
    }
    if let Ok(commit) = head.peel_to_commit() {
        let sha = commit.id().to_string();
        return Some(format!("(detached {})", &sha[..sha.len().min(7)]));
    }
    None
}

fn is_dirty(repo: &git2::Repository) -> Option<bool> {
    let mut statuses = git2::StatusOptions::new();
    statuses.include_untracked(true);
    statuses.exclude_submodules(true);
    statuses.recurse_ignored_dirs(false);

    let iter = repo.statuses(Some(&mut statuses)).ok()?;
    let dirty = iter.iter().any(|s| s.status() != git2::Status::CURRENT);
    Some(dirty)
}

fn ahead_behind(repo: &git2::Repository) -> Option<(u32, u32)> {
    let head = repo.head().ok()?;
    let local = head
        .shorthand()
        .and_then(|name| repo.find_branch(name, git2::BranchType::Local).ok());

    let upstream = local.and_then(|b| b.upstream().ok());
    let Some(upstream) = upstream else {
        return Some((0, 0));
    };

    let local_oid = head.target()?;
    let upstream_oid = upstream.get().target()?;
    let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid).ok()?;
    Some((ahead as u32, behind as u32))
}

fn last_commit(repo: &git2::Repository) -> Result<Option<Commit>> {
    let head = repo
        .head()
        .map_err(|e| vobes_core::Error::git(e.to_string()))?;
    let commit = head
        .peel_to_commit()
        .map_err(|e| vobes_core::Error::git(e.to_string()))?;

    let hash = commit.id().to_string();
    let message = commit.summary().unwrap_or("").to_string();
    let author = commit.author();
    let name = author.name().unwrap_or("");
    let email = author.email().unwrap_or("");
    let author_str = if email.is_empty() {
        name.to_string()
    } else {
        format!("{name} <{email}>")
    };
    let date = commit_time(&author);

    Ok(Some(Commit {
        hash,
        message,
        author: author_str,
        date,
    }))
}

fn commit_time(sig: &git2::Signature<'_>) -> DateTime<Utc> {
    let time = sig.when();
    let secs = time.seconds();
    DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fresh_repo(name: &str) -> (PathBuf, git2::Repository) {
        let dir = std::env::temp_dir().join(format!("vobes-git-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let repo = git2::Repository::init(&dir).unwrap();
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "Vobes Test").unwrap();
        cfg.set_str("user.email", "test@vobes.local").unwrap();
        (dir, repo)
    }

    fn commit(repo: &git2::Repository, dir: &Path, name: &str) -> git2::Oid {
        fs::write(dir.join(name), "x").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(name)).unwrap();
        index.write().unwrap();
        let tree = index.write_tree().unwrap();
        let sig = repo.signature().unwrap();
        let parents: Vec<git2::Commit<'_>> = match repo.head().ok() {
            Some(h) => vec![h.peel_to_commit().unwrap()],
            None => Vec::new(),
        };
        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("commit {name}"),
            &repo.find_tree(tree).unwrap(),
            &parent_refs,
        )
        .unwrap()
    }

    #[test]
    fn returns_none_for_non_repo() {
        let dir = std::env::temp_dir().join("vobes-git-test-nonrepo");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let info = read_git_info(&dir).unwrap();
        assert!(info.is_none());
    }

    #[test]
    fn reads_branch_and_last_commit() {
        let (dir, repo) = fresh_repo("basic");
        commit(&repo, &dir, "a.txt");

        let info = read_git_info(&dir).unwrap().unwrap();
        assert!(!info.branch.is_empty());
        assert_eq!(info.ahead, 0);
        assert_eq!(info.behind, 0);
        let c = info.last_commit.expect("last commit");
        assert!(!c.hash.is_empty());
        assert!(c.author.contains("Vobes Test"));
        assert_eq!(c.message, "commit a.txt");
    }

    #[test]
    fn dirty_after_unstaged_change() {
        let (dir, repo) = fresh_repo("dirty");
        commit(&repo, &dir, "a.txt");
        fs::write(dir.join("a.txt"), "changed").unwrap();

        let info = read_git_info(&dir).unwrap().unwrap();
        assert!(info.dirty, "should be dirty, got {:?}", info);
    }

    #[test]
    fn untracked_file_marks_dirty() {
        let (dir, repo) = fresh_repo("untracked");
        commit(&repo, &dir, "a.txt");
        fs::write(dir.join("b.txt"), "new").unwrap();

        let info = read_git_info(&dir).unwrap().unwrap();
        assert!(info.dirty);
    }

    #[test]
    fn clean_repo_is_clean() {
        let (dir, repo) = fresh_repo("clean");
        commit(&repo, &dir, "a.txt");

        let info = read_git_info(&dir).unwrap().unwrap();
        assert!(!info.dirty);
        assert!(info.is_clean());
    }
}
