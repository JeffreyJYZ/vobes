//! `vbs watch` — stream activity as newline-delimited JSON (NDJSON).
//!
//! Polls the store every second and prints any new activity events. One
//! JSON object per line. Stop with Ctrl-C. Intended for AI agents / pipes
//! that want a live feed without polling `vbs log` themselves.

use std::time::Duration;

use vobes_core::Result;

use vobes_cli::app::App;

pub fn run(app: &App) -> Result<()> {
    let mut last_id: u64 = app
        .store
        .recent_activity(1)?
        .into_iter()
        .filter_map(|e| e.id)
        .next()
        .unwrap_or(0);

    println!("{{\"type\":\"ready\"}}");
    loop {
        let events = app.store.recent_activity(50)?;
        for e in events {
            let id = match e.id {
                Some(i) => i,
                None => continue,
            };
            if id > last_id {
                last_id = id;
                let line = serde_json::to_string(&e)?;
                println!("{line}");
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}
