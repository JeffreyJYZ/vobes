//! `vbs list` — list tracked vobes.

use vobes_core::{Query, Result};
use vobes_store::{Filter, Sort};

use crate::output;
use vobes_cli::app::App;

pub fn run(app: &App, json: bool, query: Option<&str>) -> Result<()> {
    let all = app
        .store
        .list_vobes(&Filter::all().exclude_archived(), Sort::LastModified)?;
    // The shared query language — the same predicates the desktop search
    // and the MCP server accept.
    let vobes: Vec<vobes_core::Vobe> = match query {
        Some(q) if !q.trim().is_empty() => {
            let parsed = Query::parse(q);
            parsed.filter(all.iter()).into_iter().cloned().collect()
        }
        _ => all,
    };
    if json {
        output::print_json(&vobes)?;
    } else {
        output::render_vobe_table(&vobes);
    }
    Ok(())
}
