//! `vbs rename <name> <new-name>` — change a vobe's display name.

use vobes_core::Result;

use crate::commands::shared::lookup_vobe;
use vobes_cli::app::App;

pub fn run(app: &App, name: &str, new_name: &str) -> Result<()> {
    let Some(vobe) = lookup_vobe(app, name)? else {
        return Err(vobes_core::Error::not_found(name.to_string()));
    };
    let renamed = app.store.rename_vobe(&vobe.id, new_name)?;
    println!("renamed: {} -> {}", vobe.name, renamed.name);
    Ok(())
}
