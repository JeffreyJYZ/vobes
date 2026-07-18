//! Tests for `Filter`, `Sort`, and `ExportSnapshot` shape.

use chrono::Utc;
use vobes_core::{ActivityEvent, ActivityKind, Vobe};
use vobes_store::{ExportSnapshot, Filter, Sort};

#[test]
fn filter_defaults_match_everything() {
    let f = Filter::all();
    assert!(f.tag.is_none());
    assert!(!f.only_dirty);
    assert!(!f.exclude_archived);
    assert!(f.modified_since.is_none());
}

#[test]
fn filter_builders_compose() {
    let f = Filter::all()
        .with_tag("work")
        .only_dirty()
        .exclude_archived()
        .modified_since(Utc::now());
    assert_eq!(f.tag.as_deref(), Some("work"));
    assert!(f.only_dirty);
    assert!(f.exclude_archived);
    assert!(f.modified_since.is_some());
}

#[test]
fn sort_default_is_last_modified() {
    let s = Sort::default();
    assert!(matches!(s, Sort::LastModified));
}

#[test]
fn snapshot_includes_version_and_current() {
    let vobes = vec![Vobe::new("demo", "/tmp/demo")];
    let id = vobes[0].id.clone();
    let activity = vec![ActivityEvent::now(id, ActivityKind::Created)];
    let snap = ExportSnapshot::new(vobes, activity);
    assert_eq!(snap.version, ExportSnapshot::CURRENT_VERSION);
    assert_eq!(snap.vobes.len(), 1);
    assert_eq!(snap.activity.len(), 1);
}

#[test]
fn snapshot_serializes_with_expected_shape() {
    let vobes = vec![Vobe::new("demo", "/tmp/demo")];
    let snap = ExportSnapshot::new(vobes, Vec::new());
    let s = serde_json::to_string(&snap).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["version"], serde_json::json!(1));
    assert!(v["exported_at"].is_string());
    assert!(v["vobes"].is_array());
    assert!(v["activity"].is_array());
}
