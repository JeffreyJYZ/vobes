//! `vbs show <name>` — inspect one vobe in detail.

use vobes_core::Result;

use crate::commands::shared::lookup_vobe;
use crate::output;
use vobes_cli::app::App;

pub fn run(app: &App, name: &str, json: bool) -> Result<()> {
    let Some(vobe) = lookup_vobe(app, name)? else {
        return Err(vobes_core::Error::not_found(name.to_string()));
    };
    if json {
        output::print_json(&vobe)?;
        return Ok(());
    }
    output::render_vobe_detail(&vobe);

    // Also show recent activity for this vobe.
    let activity = app.store.vobe_activity(&vobe.id, 10)?;
    println!();
    println!("recent activity:");
    output::render_activity(&activity, |_| Some(vobe.name.clone()));
    Ok(())
}
