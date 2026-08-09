//! Vobes storage crate — SQLite primary, JSON export.
//!
//! The `Store` trait is the stable interface consumed by platform crates.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

mod json;
mod model;
mod schema;
mod sqlite;

pub use model::{ExportSnapshot, Filter, SavedFilter, Sort};
pub use sqlite::SqliteStore;
pub use vobes_core::{ActivityEvent, Result, Vobe, VobeId};

use std::path::Path;

/// Store trait — the stable interface platform crates consume.
pub trait Store: Send + Sync {
    /// Insert or update a vobe.
    fn upsert_vobe(&self, vobe: &Vobe) -> Result<()>;
    /// Fetch a single vobe by id.
    fn get_vobe(&self, id: &VobeId) -> Result<Option<Vobe>>;
    /// Fetch a single vobe by name.
    fn get_vobe_by_name(&self, name: &str) -> Result<Option<Vobe>>;
    /// Fetch a single vobe by path.
    fn get_vobe_by_path(&self, path: &Path) -> Result<Option<Vobe>>;
    /// List vobes matching the filter, sorted.
    fn list_vobes(&self, filter: &Filter, sort: Sort) -> Result<Vec<Vobe>>;
    /// Delete a vobe (cascades to activity).
    fn delete_vobe(&self, id: &VobeId) -> Result<()>;
    /// Record an activity event.
    fn record_activity(&self, event: &ActivityEvent) -> Result<()>;
    /// Most recent N events globally (newest first).
    fn recent_activity(&self, limit: usize) -> Result<Vec<ActivityEvent>>;
    /// Most recent N events for a vobe (newest first).
    fn vobe_activity(&self, vobe_id: &VobeId, limit: usize) -> Result<Vec<ActivityEvent>>;
    /// Most recent N events filtered by the `actor` label (newest
    /// first). Returns all actors when `actor` is `None`. Backs the
    /// activity-feed "agent vs human" filter in the desktop UI.
    fn recent_activity_by_actor(
        &self,
        actor: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ActivityEvent>> {
        let mut events = self.recent_activity(limit)?;
        if let Some(actor) = actor {
            events.retain(|e| e.actor == actor);
        }
        Ok(events)
    }
    /// Export all data as JSON to the given path.
    fn export_json(&self, path: &Path) -> Result<()>;
    /// Import data from a previous JSON export.
    fn import_json(&self, path: &Path) -> Result<()>;
    /// Delete every vobe and all activity. Dangerous — used by reset.
    fn purge_all(&self) -> Result<()>;
    /// List all saved filters, newest first.
    fn list_saved_filters(&self) -> Result<Vec<SavedFilter>>;
    /// Insert or replace a saved filter by id.
    fn upsert_saved_filter(&self, filter: &SavedFilter) -> Result<()>;
    /// Delete a saved filter by id.
    fn delete_saved_filter(&self, id: &str) -> Result<()>;
}

/// Blanket impl so `Arc<dyn Store>` / `Box<dyn Store>` can be used as a
/// `Store` directly via deref.
impl<S: Store + ?Sized> Store for std::sync::Arc<S> {
    fn upsert_vobe(&self, vobe: &Vobe) -> Result<()> {
        (**self).upsert_vobe(vobe)
    }
    fn get_vobe(&self, id: &VobeId) -> Result<Option<Vobe>> {
        (**self).get_vobe(id)
    }
    fn get_vobe_by_name(&self, name: &str) -> Result<Option<Vobe>> {
        (**self).get_vobe_by_name(name)
    }
    fn get_vobe_by_path(&self, path: &Path) -> Result<Option<Vobe>> {
        (**self).get_vobe_by_path(path)
    }
    fn list_vobes(&self, filter: &Filter, sort: Sort) -> Result<Vec<Vobe>> {
        (**self).list_vobes(filter, sort)
    }
    fn delete_vobe(&self, id: &VobeId) -> Result<()> {
        (**self).delete_vobe(id)
    }
    fn record_activity(&self, event: &ActivityEvent) -> Result<()> {
        (**self).record_activity(event)
    }
    fn recent_activity(&self, limit: usize) -> Result<Vec<ActivityEvent>> {
        (**self).recent_activity(limit)
    }
    fn vobe_activity(&self, vobe_id: &VobeId, limit: usize) -> Result<Vec<ActivityEvent>> {
        (**self).vobe_activity(vobe_id, limit)
    }
    fn recent_activity_by_actor(
        &self,
        actor: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ActivityEvent>> {
        (**self).recent_activity_by_actor(actor, limit)
    }
    fn export_json(&self, path: &Path) -> Result<()> {
        (**self).export_json(path)
    }
    fn import_json(&self, path: &Path) -> Result<()> {
        (**self).import_json(path)
    }
    fn purge_all(&self) -> Result<()> {
        (**self).purge_all()
    }
    fn list_saved_filters(&self) -> Result<Vec<SavedFilter>> {
        (**self).list_saved_filters()
    }
    fn upsert_saved_filter(&self, filter: &SavedFilter) -> Result<()> {
        (**self).upsert_saved_filter(filter)
    }
    fn delete_saved_filter(&self, id: &str) -> Result<()> {
        (**self).delete_saved_filter(id)
    }
}
