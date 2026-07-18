//! `vbs init` — create a default `vobes.toml` in the user config dir.

use std::fs;

use vobes_core::Result;

use crate::app::App;

const DEFAULT_CONFIG: &str = r#"# Vobes configuration. Edit and rerun `vbs scan`.

[general]
name = "Personal Workspace"

[scan]
roots = ["~/dev", "~/work"]
exclude = ["scratch", "experiments", "tmp"]
max_depth = 4
follow_symlinks = false

[display]
theme = "auto"
date_format = "relative"
default_sort = "last_modified"

[git]
cache_ttl_seconds = 60
fetch_upstream = false

[export]
path = "~/.vobes/snapshots"
format = "json"
"#;

pub fn run(app: &App) -> Result<()> {
    let Some(path) = &app.config_path else {
        return Err(vobes_core::Error::config(
            "cannot resolve platform config dir for vobes.toml".to_string(),
        ));
    };
    if path.exists() {
        eprintln!("config already exists: {}", path.display());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| vobes_core::Error::config(format!("create dir: {e}")))?;
    }
    fs::write(path, DEFAULT_CONFIG)
        .map_err(|e| vobes_core::Error::config(format!("write: {e}")))?;
    println!("created: {}", path.display());
    Ok(())
}
