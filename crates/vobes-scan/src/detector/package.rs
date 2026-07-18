//! Package manager detector — identifies the primary package manager
//! by lockfile presence.

use std::path::Path;
use vobes_core::Result;

use crate::detector::{Detection, Detector};

/// Known lockfiles and the package manager they signal.
///
/// Order matters: we check earlier entries first, so a project with
/// both `pnpm-lock.yaml` and `package-lock.json` reports pnpm.
const LOCKFILES: &[(&str, &str)] = &[
    ("pnpm-lock.yaml", "pnpm"),
    ("yarn.lock", "yarn"),
    ("package-lock.json", "npm"),
    ("bun.lockb", "bun"),
    ("bun.lock", "bun"),
    ("Cargo.lock", "cargo"),
    ("poetry.lock", "poetry"),
    ("Pipfile.lock", "pipenv"),
    ("uv.lock", "uv"),
    ("go.sum", "go"),
    ("go.mod", "go"),
    ("Gemfile.lock", "bundler"),
    ("mix.lock", "hex"),
    ("composer.lock", "composer"),
    ("pubspec.lock", "pub"),
    ("pubspec.yaml", "pub"),
    ("Project.toml", "julia"),
    ("Manifest.toml", "julia"),
    ("flake.lock", "nix"),
    ("flake.nix", "nix"),
    ("podfile.lock", "cocoapods"),
    ("Package.resolved", "swift"),
];

/// Detects the primary package manager of a directory.
#[derive(Debug, Default, Clone, Copy)]
pub struct PackageManagerDetector;

impl PackageManagerDetector {
    /// Create a new package manager detector.
    pub fn new() -> Self {
        Self
    }
}

impl Detector for PackageManagerDetector {
    fn name(&self) -> &str {
        "package_manager"
    }

    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        if !path.is_dir() {
            return Ok(None);
        }
        for (file, pm) in LOCKFILES {
            if path.join(file).exists() {
                return Ok(Some(Detection {
                    package_manager: Some((*pm).to_string()),
                    ..Default::default()
                }));
            }
        }
        Ok(None)
    }
}
