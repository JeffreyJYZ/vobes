//! Language detector — identifies the primary language by file
//! extension census, ignoring built-in junk directories.

use std::collections::HashMap;
use std::path::Path;
use vobes_core::Result;
use walkdir::WalkDir;

use crate::detector::{Detection, Detector};
use crate::exclude::is_excluded;

/// Map of file extension → language name, for the most common cases.
///
/// Order matters only for tie-breaking. Multiple extensions (`ts`, `tsx`)
/// map to TypeScript so React/Next projects show TS even with JSX files.
fn language_for_ext(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "rs" => "Rust",
        "ts" | "tsx" | "mts" | "cts" => "TypeScript",
        "js" | "jsx" | "mjs" | "cjs" => "JavaScript",
        "py" | "pyi" => "Python",
        "go" => "Go",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "swift" => "Swift",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" | "hh" => "C++",
        "cs" => "C#",
        "rb" => "Ruby",
        "php" => "PHP",
        "scala" | "sc" => "Scala",
        "clj" | "cljs" | "cljc" | "edn" => "Clojure",
        "ex" | "exs" => "Elixir",
        "erl" => "Erlang",
        "hs" => "Haskell",
        "ml" | "mli" => "OCaml",
        "fs" | "fsx" => "F#",
        "lua" => "Lua",
        "dart" => "Dart",
        "zig" => "Zig",
        "nim" => "Nim",
        "jl" => "Julia",
        "r" => "R",
        "sh" | "bash" | "zsh" => "Shell",
        "ps1" => "PowerShell",
        "bat" | "cmd" => "Batch",
        "vue" => "Vue",
        "svelte" => "Svelte",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "less" => "Less",
        "sql" => "SQL",
        "elm" => "Elm",
        "purs" => "PureScript",
        "cr" => "Crystal",
        "d" => "D",
        "vala" => "Vala",
        "vim" => "Vimscript",
        "tex" => "LaTeX",
        _ => return None,
    })
}

/// Detects the primary language of a directory by counting file
/// extensions up to a depth limit and a max-file budget.
#[derive(Debug, Clone)]
pub struct LanguageDetector {
    /// Maximum number of files to sample (default 4096).
    pub max_files: usize,
    /// Maximum walk depth below the root (default 6).
    pub max_depth: usize,
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self {
            max_files: 4096,
            max_depth: 6,
        }
    }
}

impl LanguageDetector {
    /// Create with sensible defaults.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Detector for LanguageDetector {
    fn name(&self) -> &str {
        "language"
    }

    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        if !path.is_dir() {
            return Ok(None);
        }
        let mut counts: HashMap<&'static str, usize> = HashMap::new();
        let mut seen_files: usize = 0;

        for entry in WalkDir::new(path)
            .max_depth(self.max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if !e.file_type().is_dir() {
                    return true;
                }
                e.file_name()
                    .to_str()
                    .map(|n| !is_excluded(n, &[]))
                    .unwrap_or(true)
            })
        {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_file() {
                continue;
            }
            if seen_files >= self.max_files {
                break;
            }
            seen_files += 1;
            let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if let Some(lang) = language_for_ext(ext) {
                *counts.entry(lang).or_default() += 1;
            }
        }

        if counts.is_empty() {
            return Ok(None);
        }
        let primary = counts
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map(|(l, _)| l.to_string());
        Ok(primary.map(|language| Detection {
            language: Some(language),
            ..Default::default()
        }))
    }
}
