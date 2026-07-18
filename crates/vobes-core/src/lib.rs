//! Vobes shared domain models, traits, and error types.
//!
//! Pure models. No IO, no filesystem, no network.
//! Consumed by all platform crates (CLI, desktop, future MCP).

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

pub mod activity;
pub mod error;
pub mod git;
pub mod vobe;

pub use activity::{ActivityEvent, ActivityKind};
pub use error::{Error, Result, VobeId};
pub use git::{Commit, GitInfo};
pub use vobe::Vobe;

/// Re-export chrono DateTime for downstream convenience.
pub use chrono::{DateTime, Utc};
