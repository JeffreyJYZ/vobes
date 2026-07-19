//! `vobes-mcp` — a Model Context Protocol server for Vobes.
//!
//! Speaks JSON-RPC 2.0 over stdio (the MCP stdio transport). No external
//! MCP SDK: the protocol is small enough to implement directly, which
//! keeps the dependency surface minimal. Exposes read-only tools that let
//! an AI agent query tracked projects, git state, and activity.

#![forbid(unsafe_code)]

use std::io::{BufRead, BufReader, Write};

use serde_json::{json, Value};
use vobes_cli::app::App;
use vobes_core::{normalize, ActivityEvent, Vobe};
use vobes_store::{Filter, Sort, Store};

fn main() -> std::process::ExitCode {
    let app = match App::load() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("vobes-mcp: failed to load app: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut out = stdout.lock();

    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(resp) = handle(&app, &request) {
            let s = serde_json::to_string(&resp).unwrap_or_default();
            let _ = writeln!(out, "{s}");
            let _ = out.flush();
        }
    }
    std::process::ExitCode::SUCCESS
}

/// Handle one JSON-RPC request. Returns `None` for notifications
/// (which expect no response).
fn handle(app: &App, req: &Value) -> Option<Value> {
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = req.get("id").cloned();

    // Notifications (no id) — acknowledge by not replying.
    if id.is_none() {
        match method {
            "notifications/initialized" | "initialized" => {}
            "notifications/cancelled" => {}
            _ => {}
        }
        return None;
    }

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "vobes", "version": "0.1.0" }
            }
        })),
        "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": tools() }
        })),
        "tools/call" => Some(handle_tool_call(app, req, id)),
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("method not found: {method}") }
        })),
    }
}

fn tools() -> Vec<Value> {
    vec![
        tool(
            "vobes_list",
            "List all tracked Vobes (projects) with git and metadata.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "vobes_show",
            "Get full detail for one Vobe by name or path.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Vobe name or absolute/relative path" }
                },
                "required": ["name"]
            }),
        ),
        tool(
            "vobes_search",
            "Search Vobes by case-insensitive name substring.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Substring to match against vobe names" }
                },
                "required": ["query"]
            }),
        ),
        tool(
            "vobes_recent_activity",
            "Return the most recent activity events across all Vobes.",
            json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "description": "Max events (default 20)", "default": 20 }
                }
            }),
        ),
        tool(
            "vobes_context",
            "Return a self-contained context pack for one Vobe: full record, recent activity, and top-level directory entries.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Vobe name or path" }
                },
                "required": ["name"]
            }),
        ),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

fn handle_tool_call(app: &App, req: &Value, id: Option<Value>) -> Value {
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(Value::Null);

    let result = match name {
        "vobes_list" => tool_list(app),
        "vobes_show" => tool_show(app, &args),
        "vobes_search" => tool_search(app, &args),
        "vobes_recent_activity" => tool_recent(app, &args),
        "vobes_context" => tool_context(app, &args),
        other => Err(format!("unknown tool: {other}")),
    };

    match result {
        Ok(text) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "content": [ { "type": "text", "text": text } ], "isError": false }
        }),
        Err(msg) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "content": [ { "type": "text", "text": msg } ], "isError": true }
        }),
    }
}

fn lookup(app: &App, name: &str) -> Result<Option<Vobe>, String> {
    if let Some(v) = app
        .store
        .get_vobe_by_name(name)
        .map_err(|e| e.to_string())?
    {
        return Ok(Some(v));
    }
    let abs = normalize(&std::path::PathBuf::from(name));
    app.store
        .get_vobe_by_path(&abs)
        .map_err(|e| e.to_string())
}

fn tool_list(app: &App) -> Result<String, String> {
    let vobes = app
        .store
        .list_vobes(&Filter::all().exclude_archived(), Sort::LastModified)
        .map_err(|e| e.to_string())?;
    serialize(&vobes)
}

fn tool_show(app: &App, args: &Value) -> Result<String, String> {
    let name = arg_str(args, "name")?;
    let Some(vobe) = lookup(app, &name)? else {
        return Err(format!("vobe not found: {name}"));
    };
    let activity = app
        .store
        .vobe_activity(&vobe.id, 10)
        .map_err(|e| e.to_string())?;
    serialize(&json!({ "vobe": vobe, "recent_activity": activity }))
}

fn tool_search(app: &App, args: &Value) -> Result<String, String> {
    let query = arg_str(args, "query")?.to_lowercase();
    let all = app
        .store
        .list_vobes(&Filter::all(), Sort::Name)
        .map_err(|e| e.to_string())?;
    let matched: Vec<&Vobe> = all
        .iter()
        .filter(|v| v.name.to_lowercase().contains(&query))
        .collect();
    serialize(&matched)
}

fn tool_recent(app: &App, args: &Value) -> Result<String, String> {
    let limit = args
        .get("limit")
        .and_then(|l| l.as_u64())
        .map(|l| l as usize)
        .unwrap_or(20);
    let events: Vec<ActivityEvent> = app
        .store
        .recent_activity(limit)
        .map_err(|e| e.to_string())?;
    serialize(&events)
}

fn tool_context(app: &App, args: &Value) -> Result<String, String> {
    let name = arg_str(args, "name")?;
    let Some(vobe) = lookup(app, &name)? else {
        return Err(format!("vobe not found: {name}"));
    };
    let activity = app
        .store
        .vobe_activity(&vobe.id, 10)
        .map_err(|e| e.to_string())?;
    let entries = std::fs::read_dir(&vobe.path)
        .map(|iter| {
            iter.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    serialize(&json!({ "vobe": vobe, "recent_activity": activity, "entries": entries }))
}

fn arg_str(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing string argument: {key}"))
}

fn serialize<T: serde::Serialize>(v: &T) -> Result<String, String> {
    serde_json::to_string_pretty(v).map_err(|e| e.to_string())
}
