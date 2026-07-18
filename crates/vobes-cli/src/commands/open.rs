//! `vbs open <name>` — record Opened event and launch $EDITOR.

use vobes_core::{ActivityEvent, ActivityKind, Result};

use crate::app::App;
use crate::commands::shared::lookup_vobe;

pub fn run(app: &App, name: &str) -> Result<()> {
    let Some(mut vobe) = lookup_vobe(app, name)? else {
        return Err(vobes_core::Error::not_found(name.to_string()));
    };
    vobe.touch_opened();
    app.store.upsert_vobe(&vobe)?;
    app.store.record_activity(
        &ActivityEvent::now(vobe.id.clone(), ActivityKind::Opened).with_detail("vbs open"),
    )?;

    // Launch editor if configured. Best-effort.
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
    let dir = &vobe.path;
    let status = std::process::Command::new(&editor).arg(dir).status();
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(vobes_core::Error::internal(format!(
            "editor exited with {s}"
        ))),
        Err(e) => Err(vobes_core::Error::internal(format!(
            "launch editor {editor}: {e}"
        ))),
    }
}
