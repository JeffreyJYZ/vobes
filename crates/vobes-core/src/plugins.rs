//! Plugin extension points.
//!
//! Vobes is open by composition: the `Detector` trait in
//! `vobes_scan` already lets downstream code add a project
//! detector without touching the core. This module declares the
//! remaining extension surfaces — `Action`, `Widget`, and a
//! `Plugin` aggregate — so future work has a stable shape to
//! target. The traits are intentionally minimal: a real plugin
//! registry is a Phase 5+ concern; right now this module is the
//! design artifact.
//!
//! ## Shapes
//!
//! ```text
//! ┌─────────────┐     detours     ┌─────────────────────┐
//! │  Detector   │   ────────────▶ │  vobes_scan + CLI   │
//! │ (ship)      │                 │  desktop scan path  │
//! └─────────────┘                 └─────────────────────┘
//!
//! ┌─────────────┐     detours     ┌─────────────────────┐
//! │  Action     │   ────────────▶ │  command palette    │
//! │ (design)    │                 │  vbs <action>       │
//! └─────────────┘                 └─────────────────────┘
//!
//! ┌─────────────┐     detours     ┌─────────────────────┐
//! │  Widget     │   ────────────▶ │  dashboard panel    │
//! │ (design)    │                 │  projects detail    │
//! └─────────────┘                 └─────────────────────┘
//! ```
//!
//! ## Why not a dynamic registry yet
//!
//! - Vobes ships as a single Rust binary + Tauri bundle. Dynamic
//!   loading adds a security surface (loading arbitrary `.so`/
//!   `.dll`/`.dylib` into a process the user trusts) for little
//!   gain while the user base is small.
//! - Compile-time composition via these traits already covers the
//!   "fork-and-extend" path: a downstream crate implements
//!   `Plugin`, calls `DefaultScanner::new(plugin.detectors())`,
//!   and ships their own binary.
//! - The traits below are deliberately `Send + Sync` so a future
//!   dynamic registry (e.g. WASM plugins via `wasmtime`) can adopt
//!   them without a breaking change.

use std::sync::Arc;

use crate::{Result, Vobe};

/// Shell context passed to [`Action::run`].
///
/// Deliberately read-only: an action inspects the vobe (and, in the
/// future, the store) and emits side effects via the trait's own
/// return. Mutations go through the existing `Store` API, not
/// through the action surface — keeps audit trivial.
#[derive(Debug, Clone)]
pub struct ActionContext {
    /// Vobe the action was invoked on, if any.
    pub vobe: Option<Arc<Vobe>>,
    /// Free-form invocation argument (e.g. a typed query). Plugins
    /// define their own parsing for this.
    pub argument: Option<String>,
}

/// A user-runnable, palette-addressable operation.
///
/// Built-ins (`open`, `reveal`, `export`, …) match this shape; a
/// downstream plugin adds more without touching the palette code.
/// The trait is `Send + Sync` so it can live in a `Vec<Arc<dyn
/// Action>>` shared across Tauri / CLI / MCP.
///
/// # Future
///
/// A `palette_label(ctx)` hook and a `keyboard_score(ctx)` hook
/// will land when the palette ranking model stabilises.
pub trait Action: Send + Sync {
    /// Stable id, unique across all plugins. Use a reverse-DNS
    /// scheme to avoid collisions (`land.jyz.vobes.open`).
    fn id(&self) -> &str;
    /// One-line label shown in the palette.
    fn label(&self) -> &str;
    /// Longer help, shown on hover / `?`.
    fn description(&self) -> &str;
    /// Execute the action. Returning `Err` surfaces a toast and
    /// aborts any chained invocations.
    fn run(&self, ctx: &ActionContext) -> Result<()>;
}

/// A read-only panel surfaced on the Projects detail view.
///
/// Implementations render structured data (next-actions, release
/// notes, kindred-projects, …) and may emit follow-up actions via
/// the action registry. The frontend receives a serialised
/// [`WidgetPayload`] over the existing Tauri invoke path.
///
/// Why a trait, not Svelte components: keep the plugin boundary in
/// Rust so the same plugin works for CLI (`vbs show --widget`)
/// and MCP without a JS toolchain. A first-class Svelte component
/// registry is a later, larger design.
pub trait Widget: Send + Sync {
    /// Stable id, unique across all plugins.
    fn id(&self) -> &str;
    /// Human label shown as the panel header.
    fn label(&self) -> &str;
    /// Return the data to render. The shape is plugin-defined;
    /// the frontend must have a matching renderer keyed on
    /// `id`.
    fn render(&self, vobe: &Vobe) -> Result<WidgetPayload>;
}

/// Payload a [`Widget`] returns. Kept opaque (a JSON blob) so the
/// plugin and frontend agree on schema out-of-band — same pattern
/// as MCP tool results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WidgetPayload {
    /// Plugin-defined JSON.
    pub body: serde_json::Value,
}

/// Aggregate plugin descriptor. A downstream crate implements this
/// once and calls [`Plugin::register`] to hand the pieces to the
/// Vobes runtime. The built-in "plugin" is constructed internally
/// and supplies the four stock detectors + the standard actions;
/// third parties extend by composing a `Plugin` with their own
/// extras.
///
/// # Example (sketch)
///
/// ```no_run
/// use vobes_core::plugins::{Action, ActionContext, Plugin, Widget};
/// # struct MyAction;
/// # impl Action for MyAction {
/// #     fn id(&self) -> &str { "land.example.my-action" }
/// #     fn label(&self) -> &str { "Do the thing" }
/// #     fn description(&self) -> &str { "" }
/// #     fn run(&self, _: &ActionContext) -> vobes_core::Result<()> { Ok(()) }
/// # }
/// # struct MyWidget;
/// # impl Widget for MyWidget {
/// #     fn id(&self) -> &str { "land.example.my-widget" }
/// #     fn label(&self) -> &str { "My widget" }
/// #     fn render(&self, _: &vobes_core::Vobe)
/// #         -> vobes_core::Result<vobes_core::plugins::WidgetPayload>
/// #     { unreachable!() }
/// # }
/// struct MyPlugin;
/// impl Plugin for MyPlugin {
///     fn id(&self) -> &str { "land.example.my-plugin" }
///     fn actions(&self) -> Vec<Box<dyn Action>> { vec![Box::new(MyAction)] }
///     fn widgets(&self) -> Vec<Box<dyn Widget>> { vec![Box::new(MyWidget)] }
///     fn register(&self) { /* hand boxes to the Vobes runtime */ }
/// }
/// ```
pub trait Plugin: Send + Sync {
    /// Stable plugin id.
    fn id(&self) -> &str;
    /// Actions this plugin contributes.
    fn actions(&self) -> Vec<Box<dyn Action>>;
    /// Widgets this plugin contributes.
    fn widgets(&self) -> Vec<Box<dyn Widget>>;
    /// Hook called by the Vobes runtime at startup — the plugin is
    /// expected to push its actions/widgets into the registries
    /// supplied by the host. Currently a no-op stub; the registry
    /// crate will land alongside `vobes-plugins` later in Phase 5+.
    fn register(&self);
}

/// Sentinel marker for the built-in plugin. Used by the runtime to
/// distinguish user-supplied plugins from the stock `vobes` actions
/// when deduping ids. Not stable — current id is the crate's own.
pub const BUILTIN_PLUGIN_ID: &str = "land.jyz.vobes.builtin";