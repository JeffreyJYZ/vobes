//! `vbs log` — show activity timeline.

use std::collections::HashMap;

use vobes_core::Result;
use vobes_store::Sort;

use crate::output;
use vobes_cli::app::App;

pub fn run(app: &App, limit: usize, json: bool) -> Result<()> {
    let vobes = app
        .store
        .list_vobes(&vobes_store::Filter::all(), Sort::Name)?;
    let names: HashMap<_, _> = vobes
        .iter()
        .map(|v| (v.id.clone(), v.name.clone()))
        .collect();
    let events = app.store.recent_activity(limit)?;
    if json {
        output::print_json(&events)?;
    } else {
        output::render_activity(&events, |id| names.get(id).cloned());
    }
    Ok(())
}
