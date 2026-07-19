//! The default scanner — walks roots with `walkdir`, runs detectors in
//! parallel with `rayon`, and produces `ScanReport`s.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;
use vobes_core::Result;
use walkdir::WalkDir;

use crate::detector::{Detection, Detector, Scanner};
use crate::exclude::{is_code_subdir, is_excluded};
use crate::{FrameworkDetector, LanguageDetector, PackageManagerDetector, RepoDetector};

/// Output of scanning a single root.
#[derive(Debug, Clone, Default)]
pub struct ScanReport {
    /// Root that was scanned.
    pub root: PathBuf,
    /// `(path, detection)` pairs for every candidate vobe found.
    pub vobes: Vec<(PathBuf, Detection)>,
    /// Number of directories walked.
    pub dirs_walked: usize,
    /// Number of directories skipped because of excludes.
    pub dirs_skipped: usize,
}

impl ScanReport {
    /// Total number of vobes discovered.
    pub fn count(&self) -> usize {
        self.vobes.len()
    }

    /// Whether any vobes were discovered.
    pub fn is_empty(&self) -> bool {
        self.vobes.is_empty()
    }
}

/// Default scanner composing a set of detectors.
#[derive(Clone)]
pub struct DefaultScanner {
    detectors: Arc<Vec<Arc<dyn Detector>>>,
    /// User-supplied extra exclude names (in addition to [`crate::BUILTIN_EXCLUDES`]).
    extra_excludes: Vec<String>,
    /// Maximum walk depth below each root.
    max_depth: usize,
    /// Whether to follow symlinks while walking.
    follow_symlinks: bool,
}

impl DefaultScanner {
    /// Build a scanner with the standard detector set (repo, language,
    /// package manager, framework).
    pub fn with_standard_detectors() -> Self {
        let detectors: Vec<Arc<dyn Detector>> = vec![
            Arc::new(RepoDetector::new()),
            Arc::new(LanguageDetector::new()),
            Arc::new(PackageManagerDetector::new()),
            Arc::new(FrameworkDetector::new()),
        ];
        Self::new(detectors)
    }

    /// Build a scanner with a custom detector set.
    pub fn new(detectors: Vec<Arc<dyn Detector>>) -> Self {
        Self {
            detectors: Arc::new(detectors),
            extra_excludes: Vec::new(),
            max_depth: 4,
            follow_symlinks: false,
        }
    }

    /// Add extra exclude names.
    pub fn with_extra_excludes(mut self, excludes: Vec<String>) -> Self {
        self.extra_excludes = excludes;
        self
    }

    /// Set the maximum walk depth below each root.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Enable or disable symlink following during walks.
    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }
}

impl Scanner for DefaultScanner {
    fn scan(&self, root: &Path) -> Result<Vec<(PathBuf, Detection)>> {
        Ok(self.scan_report(root)?.vobes)
    }
}

impl DefaultScanner {
    /// Scan a root and return a full [`ScanReport`].
    pub fn scan_report(&self, root: &Path) -> Result<ScanReport> {
        let extra = self.extra_excludes.clone();
        let detectors = self.detectors.clone();

        let mut dirs_walked = 0usize;
        let mut dirs_skipped = 0usize;
        let mut candidates: Vec<PathBuf> = Vec::new();

        for entry in WalkDir::new(root)
            .max_depth(self.max_depth)
            .follow_links(self.follow_symlinks)
            .into_iter()
            .filter_entry(|e| {
                if !e.file_type().is_dir() {
                    return true;
                }
                let name = e.file_name();
                let Some(n) = name.to_str() else {
                    return true;
                };
                if is_excluded(n, &extra) {
                    dirs_skipped += 1;
                    return false;
                }
                true
            })
        {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_dir() {
                continue;
            }
            dirs_walked += 1;
            // Source-only subdirs (src, src-tauri, lib, app) are not
            // themselves projects, but we still descend so the parent
            // project's detectors can see their files.
            let is_code = entry
                .file_name()
                .to_str()
                .map(is_code_subdir)
                .unwrap_or(false);
            if is_code {
                continue;
            }
            candidates.push(entry.path().to_path_buf());
        }

        // Run detectors in parallel across candidate directories.
        let detections: Vec<(PathBuf, Detection)> = candidates
            .par_iter()
            .filter_map(|path| {
                let mut merged = Detection::empty();
                let mut any = false;
                for d in detectors.iter() {
                    if let Ok(Some(det)) = d.detect(path) {
                        merged.merge(det);
                        any = true;
                    }
                }
                any.then(|| (path.clone(), merged))
            })
            .collect();

        // Keep only directories that look like project roots: either a
        // git repo, or any detector produced a signal. We already filtered
        // to `any == true`. Additionally, drop empty roots that only
        // contain other projects (e.g. `~/dev` itself).
        let vobes: Vec<(PathBuf, Detection)> = detections
            .into_iter()
            .filter(|(_, d)| d.is_repo || d.framework.is_some() || d.package_manager.is_some())
            .collect();

        Ok(ScanReport {
            root: root.to_path_buf(),
            vobes,
            dirs_walked,
            dirs_skipped,
        })
    }
}
