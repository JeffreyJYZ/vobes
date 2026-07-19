//! Library surface for the Vobes CLI.
//!
//! The CLI binary (`vbs`) lives in `main.rs`; this crate also exposes its
//! app context so other tools (e.g. the MCP server) can reuse the same
//! config/store wiring without duplicating it.

pub mod app;
