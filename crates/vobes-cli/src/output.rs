//! Terminal output formatting.

use chrono::{DateTime, Utc};
use comfy_table::{ContentArrangement, Table};
use serde::Serialize;

use vobes_core::{ActivityEvent, Vobe};

/// Format a relative-time string ("3 days ago").
pub fn relative(ts: DateTime<Utc>) -> String {
    let now = Utc::now();
    let dur = now.signed_duration_since(ts);
    let secs = dur.num_seconds();
    if secs < 60 {
        return "just now".to_string();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{hours}h ago");
    }
    let days = hours / 24;
    if days < 30 {
        return format!("{days}d ago");
    }
    let months = days / 30;
    if months < 12 {
        return format!("{months}mo ago");
    }
    let years = months / 12;
    format!("{years}y ago")
}

/// Render a compact listing of vobes as a table.
pub fn render_vobe_table(vobes: &[Vobe]) {
    if vobes.is_empty() {
        println!("No vobes tracked. Run `vbs scan` to discover projects.");
        return;
    }
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Disabled)
        .set_header(vec!["Name", "Lang", "PM", "Branch", "Status", "Modified"]);

    for v in vobes {
        let lang = v.language.clone().unwrap_or_else(|| "-".into());
        let pm = v.package_manager.clone().unwrap_or_else(|| "-".into());
        let branch = v
            .git
            .as_ref()
            .map(|g| g.branch.clone())
            .unwrap_or_else(|| "-".into());
        let status = vobe_status(v);
        let modified = v.last_modified.map(relative).unwrap_or_else(|| "-".into());
        table.add_row(vec![v.name.clone(), lang, pm, branch, status, modified]);
    }
    println!("{table}");
}

fn vobe_status(v: &Vobe) -> String {
    let mut bits = Vec::new();
    if v.is_dirty() {
        bits.push("dirty".into());
    }
    if v.has_unpushed() {
        bits.push(format!("↑{}", v.git.as_ref().unwrap().ahead));
    }
    if v.has_unpulled() {
        bits.push(format!("↓{}", v.git.as_ref().unwrap().behind));
    }
    if v.is_archived() {
        bits.push("archived".into());
    }
    if bits.is_empty() {
        "clean".into()
    } else {
        bits.join(" ")
    }
}

/// Render a detailed view of a single vobe.
pub fn render_vobe_detail(v: &Vobe) {
    println!("{}", v.name);
    println!("  path:   {}", v.path.display());
    println!("  id:     {}", v.id);
    if let Some(l) = &v.language {
        println!("  lang:   {l}");
    }
    if let Some(fw) = &v.framework {
        println!("  fw:     {fw}");
    }
    if let Some(pm) = &v.package_manager {
        println!("  pm:     {pm}");
    }
    if !v.tags.is_empty() {
        println!("  tags:   {}", v.tags.join(", "));
    }
    println!("  created: {}", relative(v.created_at));
    if let Some(o) = v.last_opened {
        println!("  opened:  {}", relative(o));
    }
    if let Some(m) = v.last_modified {
        println!("  modified: {}", relative(m));
    }
    if let Some(notes) = &v.notes {
        println!("  notes:   {notes}");
    }
    if let Some(g) = &v.git {
        println!("  git:");
        println!("    branch: {}", g.branch);
        println!("    dirty:  {}", g.dirty);
        if g.ahead > 0 || g.behind > 0 {
            println!("    ahead/behind: ↑{} ↓{}", g.ahead, g.behind);
        }
        if let Some(c) = &g.last_commit {
            println!("    last:   {} {}", c.short_hash(), c.short_message(60));
            println!("            {} ({})", c.author, relative(c.date));
        }
    }
}

/// Serialize any value as pretty JSON to stdout.
pub fn print_json<T: Serialize>(value: &T) -> vobes_core::Result<()> {
    let s = serde_json::to_string_pretty(value)?;
    println!("{s}");
    Ok(())
}

/// Render an activity timeline.
pub fn render_activity(
    events: &[ActivityEvent],
    vobe_lookup: impl Fn(&vobes_core::VobeId) -> Option<String>,
) {
    if events.is_empty() {
        println!("No activity yet.");
        return;
    }
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
        .set_header(vec!["When", "Vobe", "Kind", "Detail"]);

    for e in events {
        let name = vobe_lookup(&e.vobe_id).unwrap_or_else(|| e.vobe_id.to_string());
        let detail = e.detail.clone().unwrap_or_default();
        table.add_row(vec![
            relative(e.timestamp),
            name,
            e.kind.to_string(),
            detail,
        ]);
    }
    println!("{table}");
}
