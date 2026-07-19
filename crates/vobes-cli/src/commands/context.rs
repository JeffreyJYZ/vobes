//! `vbs context <name>` — dump a compact context pack as JSON.
//!
//! Designed for AI agents: one self-contained object with the full vobe
//! plus its recent activity and a list of top-level file entries so an
//! agent can orient without a filesystem walk.

use vobes_core::Result;

use crate::commands::shared::lookup_vobe;
use vobes_cli::app::App;

#[derive(serde::Serialize)]
struct ContextPack {
    vobe: vobes_core::Vobe,
    recent_activity: Vec<vobes_core::ActivityEvent>,
    entries: Vec<String>,
}

pub fn run(app: &App, name: &str) -> Result<()> {
    let Some(vobe) = lookup_vobe(app, name)? else {
        return Err(vobes_core::Error::not_found(name.to_string()));
    };
    let activity = app.store.vobe_activity(&vobe.id, 10)?;
    let entries = std::fs::read_dir(&vobe.path)
        .map(|iter| {
            iter.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let pack = ContextPack {
        vobe,
        recent_activity: activity,
        entries,
    };
    crate::output::print_json(&pack)?;
    Ok(())
}
