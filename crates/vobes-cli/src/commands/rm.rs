//! `vbs rm <name>` — remove a vobe from tracking.

use vobes_core::Result;

use crate::app::App;
use crate::commands::shared::lookup_vobe;

pub fn run(app: &App, name: &str) -> Result<()> {
    let Some(vobe) = lookup_vobe(app, name)? else {
        return Err(vobes_core::Error::not_found(name.to_string()));
    };
    app.store.delete_vobe(&vobe.id)?;
    println!("removed: {}", vobe.name);
    Ok(())
}
