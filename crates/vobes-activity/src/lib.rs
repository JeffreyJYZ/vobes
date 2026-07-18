//! Vobes activity tracking — event recording and timeline queries.
//!
//! Implementation arrives in Phase 4. This crate wires `ActivityEvent`s
//! into storage surfaces.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use vobes_core::{ActivityEvent, Result, VobeId};

/// Trait for things that record activity and answer timeline queries.
pub trait Tracker: Send + Sync {
    /// Record an event.
    fn record(&self, event: &ActivityEvent) -> Result<()>;
    /// Most recent N events globally.
    fn recent(&self, limit: usize) -> Result<Vec<ActivityEvent>>;
    /// Most recent N events for a given vobe.
    fn for_vobe(&self, vobe_id: &VobeId, limit: usize) -> Result<Vec<ActivityEvent>>;
}
