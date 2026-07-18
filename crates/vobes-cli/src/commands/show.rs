//! `vbs show <name>` — inspect one vobe in detail.

use vobes_core::Result;

use crate::app::App;
use crate::commands::shared::lookup_vobe;
use crate::output;

pub fn run(app: &App, name: &str) -> Result<()> {
    let Some(vobe) = lookup_vobe(app, name)? else {
        return Err(vobes_core::Error::not_found(name.to_string()));
    };
    output::render_vobe_detail(&vobe);

    // Also show recent activity for this vobe.
    let activity = app.store.vobe_activity(&vobe.id, 10)?;
    println!();
    println!("recent activity:");
    output::render_activity(&activity, |_| Some(vobe.name.clone()));
    Ok(())
}
