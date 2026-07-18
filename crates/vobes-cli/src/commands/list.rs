//! `vbs list` — list tracked vobes.

use vobes_core::Result;
use vobes_store::{Filter, Sort};

use crate::app::App;
use crate::output;

pub fn run(app: &App) -> Result<()> {
    let vobes = app
        .store
        .list_vobes(&Filter::all().exclude_archived(), Sort::LastModified)?;
    output::render_vobe_table(&vobes);
    Ok(())
}
