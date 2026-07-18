//! Vobes activity tracking — event recording and timeline queries.
//!
//! The full implementation lives in the storage layer. This crate
//! provides the trait and a simple in-memory tracker used by tests and
//! ephemeral CLI sessions.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use std::sync::Mutex;

use vobes_core::{ActivityEvent, Result, VobeId};

/// Trait for things that record activity and answer timeline queries.
pub trait Tracker: Send + Sync {
    /// Record an event. Returns the storage-assigned id when applicable.
    fn record(&self, event: &ActivityEvent) -> Result<()>;
    /// Most recent N events globally (newest first).
    fn recent(&self, limit: usize) -> Result<Vec<ActivityEvent>>;
    /// Most recent N events for a given vobe (newest first).
    fn for_vobe(&self, vobe_id: &VobeId, limit: usize) -> Result<Vec<ActivityEvent>>;
}

/// Simple in-memory tracker.
///
/// Events are stored in insertion order; queries return newest first.
#[derive(Debug, Default)]
pub struct InMemoryTracker {
    events: Mutex<Vec<ActivityEvent>>,
}

impl InMemoryTracker {
    /// Create an empty in-memory tracker.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tracker for InMemoryTracker {
    fn record(&self, event: &ActivityEvent) -> Result<()> {
        let mut events = self
            .events
            .lock()
            .map_err(|e| vobes_core::Error::internal(format!("tracker lock: {e}")))?;
        events.push(event.clone());
        Ok(())
    }

    fn recent(&self, limit: usize) -> Result<Vec<ActivityEvent>> {
        let events = self
            .events
            .lock()
            .map_err(|e| vobes_core::Error::internal(format!("tracker lock: {e}")))?;
        let mut out: Vec<ActivityEvent> = events.iter().rev().take(limit).cloned().collect();
        out.reverse();
        Ok(out)
    }

    fn for_vobe(&self, vobe_id: &VobeId, limit: usize) -> Result<Vec<ActivityEvent>> {
        let events = self
            .events
            .lock()
            .map_err(|e| vobes_core::Error::internal(format!("tracker lock: {e}")))?;
        let mut out: Vec<ActivityEvent> = events
            .iter()
            .rev()
            .filter(|e| &e.vobe_id == vobe_id)
            .take(limit)
            .cloned()
            .collect();
        out.reverse();
        Ok(out)
    }
}
