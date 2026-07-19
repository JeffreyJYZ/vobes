//! Vobes scanning engine.
//!
//! Walks configured roots and produces vobe candidates via modular
//! detectors. Adding a new framework or language means adding one
//! detector — no core change.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

mod detector;
mod exclude;
mod scanner;

pub use detector::{
    framework::FrameworkDetector, language::LanguageDetector, package::PackageManagerDetector,
    repo::RepoDetector, Detection, Detector, Scanner,
};
pub use exclude::{is_code_subdir, is_excluded, BUILTIN_EXCLUDES};
pub use scanner::{DefaultScanner, ScanReport};
