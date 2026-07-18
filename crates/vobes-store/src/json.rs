//! JSON export / import.

use std::path::Path;

use vobes_core::{ActivityEvent, Result, Vobe};

use crate::model::ExportSnapshot;

/// Write a JSON snapshot to disk.
pub fn export_to_file(path: &Path, vobes: &[Vobe], activity: &[ActivityEvent]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| vobes_core::Error::storage(format!("create dir: {e}")))?;
    }
    let snap = ExportSnapshot::new(vobes.to_vec(), activity.to_vec());
    let s = serde_json::to_string_pretty(&snap)
        .map_err(|e| vobes_core::Error::storage(format!("encode snapshot: {e}")))?;
    std::fs::write(path, s)
        .map_err(|e| vobes_core::Error::storage(format!("write snapshot: {e}")))?;
    Ok(())
}

/// Read a JSON snapshot from disk.
pub fn import_from_file(path: &Path) -> Result<ExportSnapshot> {
    let s = std::fs::read_to_string(path)
        .map_err(|e| vobes_core::Error::storage(format!("read snapshot: {e}")))?;
    let snap: ExportSnapshot = serde_json::from_str(&s)
        .map_err(|e| vobes_core::Error::storage(format!("parse snapshot: {e}")))?;
    Ok(snap)
}
