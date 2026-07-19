//! `vbs export` — dump all data as JSON.

use std::path::PathBuf;

use vobes_core::Result;

use vobes_cli::app::App;

pub fn run(app: &App, out: Option<&str>) -> Result<()> {
    let path = match out {
        Some(p) => {
            let pb = PathBuf::from(p);
            if pb.is_absolute() {
                pb
            } else {
                std::env::current_dir().unwrap_or_default().join(pb)
            }
        }
        None => {
            let base = vobes_config::snapshots_dir().unwrap_or_else(|| PathBuf::from("."));
            std::fs::create_dir_all(&base).ok();
            let ts = chrono::Utc::now().format("%Y-%m-%d-%H%M%S");
            base.join(format!("vobes-{ts}.json"))
        }
    };

    app.store.export_json(&path)?;
    println!("exported -> {}", path.display());
    Ok(())
}
