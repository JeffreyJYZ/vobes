//! Framework detector — identifies the primary framework from
//! dependency manifests.

use std::path::Path;
use vobes_core::Result;

use crate::detector::{Detection, Detector};

/// Detects the primary framework of a directory.
///
/// Inspector-style: it reads small manifest files (package.json,
/// Cargo.toml, pyproject.toml, go.mod) and inspects their contents for
/// known dependency signatures. Falls back to file-name heuristics for
/// ecosystems without a manifest.
#[derive(Debug, Default, Clone, Copy)]
pub struct FrameworkDetector;

impl FrameworkDetector {
    /// Create a new framework detector.
    pub fn new() -> Self {
        Self
    }

    /// Match a framework from a `package.json`-style dependency blob.
    fn match_js(deps: &serde_json::Map<String, serde_json::Value>) -> Option<&'static str> {
        // Order matters — more specific frameworks first.
        let signatures: &[(&str, &str, &[&str])] = &[
            ("next", "Next.js", &["next"]),
            ("nuxt", "Nuxt", &["nuxt"]),
            ("remix", "Remix", &["remix", "@remix-run/react"]),
            ("sveltekit", "SvelteKit", &["@sveltejs/kit"]),
            ("astro", "Astro", &["astro"]),
            ("vite", "Vite", &["vite"]),
            ("svelte", "Svelte", &["svelte"]),
            ("solid", "Solid", &["solid-js"]),
            ("react", "React", &["react", "react-dom"]),
            ("vue", "Vue", &["vue"]),
            ("angular", "Angular", &["@angular/core"]),
            ("express", "Express", &["express"]),
            ("fastify", "Fastify", &["fastify"]),
            ("nest", "NestJS", &["@nestjs/core"]),
            ("hono", "Hono", &["hono"]),
            ("gatsby", "Gatsby", &["gatsby"]),
            ("docusaurus", "Docusaurus", &["@docusaurus/core"]),
            ("storybook", "Storybook", &["storybook"]),
        ];
        for (_, label, keys) in signatures {
            if keys.iter().any(|k| deps.contains_key(*k)) {
                return Some(label);
            }
        }
        None
    }
}

impl Detector for FrameworkDetector {
    fn name(&self) -> &str {
        "framework"
    }

    fn detect(&self, path: &Path) -> Result<Option<Detection>> {
        if !path.is_dir() {
            return Ok(None);
        }
        // package.json
        let pkg_path = path.join("package.json");
        if pkg_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&pkg_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                    for field in ["dependencies", "devDependencies", "peerDependencies"] {
                        if let Some(map) = v.get(field).and_then(|f| f.as_object()) {
                            if let Some(fw) = Self::match_js(map) {
                                return Ok(Some(Detection {
                                    framework: Some(fw.to_string()),
                                    ..Default::default()
                                }));
                            }
                        }
                    }
                }
            }
            // Empty/unknown package.json still means "Node" → report as plain Node project by absence of framework
        }

        // Cargo.toml
        let cargo_path = path.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&cargo_path) {
                let lc = s.to_lowercase();
                let fw = if lc.contains("axum") {
                    "Axum"
                } else if lc.contains("actix-web") {
                    "Actix Web"
                } else if lc.contains("rocket") {
                    "Rocket"
                } else if lc.contains("warp") {
                    "Warp"
                } else if lc.contains("poem") {
                    "Poem"
                } else if lc.contains("bevy") {
                    "Bevy"
                } else if lc.contains("tauri") {
                    "Tauri"
                } else if lc.contains("iced") {
                    "Iced"
                } else if lc.contains("egui") || lc.contains("eframe") {
                    "egui"
                } else if lc.contains("leptos") {
                    "Leptos"
                } else if lc.contains("dioxus") {
                    "Dioxus"
                } else {
                    "Rust (no framework)"
                };
                return Ok(Some(Detection {
                    framework: Some(fw.to_string()),
                    ..Default::default()
                }));
            }
        }

        // pyproject.toml
        let py_path = path.join("pyproject.toml");
        if py_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&py_path) {
                let lc = s.to_lowercase();
                let fw = if lc.contains("fastapi") {
                    "FastAPI"
                } else if lc.contains("django") {
                    "Django"
                } else if lc.contains("flask") {
                    "Flask"
                } else if lc.contains("starlette") {
                    "Starlette"
                } else if lc.contains("litestar") {
                    "Litestar"
                } else if lc.contains("poetry") {
                    "Python (Poetry)"
                } else {
                    "Python"
                };
                return Ok(Some(Detection {
                    framework: Some(fw.to_string()),
                    ..Default::default()
                }));
            }
        }

        // go.mod
        let go_path = path.join("go.mod");
        if go_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&go_path) {
                let lc = s.to_lowercase();
                let fw = if lc.contains("github.com/gin-gonic/gin") {
                    "Gin"
                } else if lc.contains("labstack/echo") {
                    "Echo"
                } else if lc.contains("gofiber/fiber") {
                    "Fiber"
                } else if lc.contains("go-chi/chi") {
                    "Chi"
                } else {
                    "Go (no framework)"
                };
                return Ok(Some(Detection {
                    framework: Some(fw.to_string()),
                    ..Default::default()
                }));
            }
        }

        // mix.exs (Elixir)
        let mix_path = path.join("mix.exs");
        if mix_path.exists() {
            let fw = if std::fs::read_to_string(&mix_path)
                .map(|s| s.to_lowercase().contains("phoenix"))
                .unwrap_or(false)
            {
                "Phoenix"
            } else {
                "Elixir (Mix)"
            };
            return Ok(Some(Detection {
                framework: Some(fw.to_string()),
                ..Default::default()
            }));
        }

        // pubspec.yaml (Flutter/Dart)
        let pub_path = path.join("pubspec.yaml");
        if pub_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&pub_path) {
                let lc = s.to_lowercase();
                let fw = if lc.contains("flutter") {
                    "Flutter"
                } else {
                    "Dart"
                };
                return Ok(Some(Detection {
                    framework: Some(fw.to_string()),
                    ..Default::default()
                }));
            }
        }

        // *.xcodeproj / Package.swift
        if path
            .read_dir()
            .map(|mut it| {
                it.any(|e| {
                    e.ok()
                        .and_then(|e| e.file_name().to_str().map(|s| s.ends_with(".xcodeproj")))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
        {
            return Ok(Some(Detection {
                framework: Some("Xcode".to_string()),
                ..Default::default()
            }));
        }
        if path.join("Package.swift").exists() {
            return Ok(Some(Detection {
                framework: Some("SwiftPM".to_string()),
                ..Default::default()
            }));
        }

        Ok(None)
    }
}
