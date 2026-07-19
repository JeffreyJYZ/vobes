//! `vbs list` — list tracked vobes.

use vobes_core::Result;
use vobes_store::{Filter, Sort};

use crate::output;
use vobes_cli::app::App;

pub fn run(app: &App, json: bool) -> Result<()> {
    let vobes = app
        .store
        .list_vobes(&Filter::all().exclude_archived(), Sort::LastModified)?;
    if json {
        output::print_json(&vobes)?;
    } else {
        output::render_vobe_table(&vobes);
    }
    Ok(())
}
