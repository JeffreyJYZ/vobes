//! `vbs log` — show activity timeline.

use std::collections::HashMap;

use vobes_core::Result;
use vobes_store::Sort;

use crate::app::App;
use crate::output;

pub fn run(app: &App, limit: usize) -> Result<()> {
    let vobes = app
        .store
        .list_vobes(&vobes_store::Filter::all(), Sort::Name)?;
    let names: HashMap<_, _> = vobes
        .iter()
        .map(|v| (v.id.clone(), v.name.clone()))
        .collect();
    let events = app.store.recent_activity(limit)?;
    output::render_activity(&events, |id| names.get(id).cloned());
    Ok(())
}
