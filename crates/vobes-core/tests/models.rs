//! Tests for the implemented `vobes-core` models.
//!
//! Only covers what's functional in Phase 1: builders, serde round-trips,
//! id handling, git helpers, activity labels. Traits and stubs get
//! exercised by their owning crates.

use chrono::Utc;
use vobes_core::{ActivityEvent, ActivityKind};
use vobes_core::{Commit, GitInfo, Vobe, VobeId};

#[test]
fn vobe_id_new_nonempty_and_unique() {
    let a = VobeId::new();
    let b = VobeId::new();
    let c = VobeId::new();
    assert!(a.as_str().starts_with("vobe_"));
    assert!(!a.as_str().is_empty());
    // Back-to-back ids must differ: the clock is not fine-grained
    // everywhere, so a counter is part of the id.
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

#[test]
fn vobe_id_serialize_as_string() {
    let id = VobeId::from_string("vobe_abc");
    let s = serde_json::to_string(&id).unwrap();
    assert_eq!(s, r#""vobe_abc""#);
    let back: VobeId = serde_json::from_str(&s).unwrap();
    assert_eq!(id, back);
}

#[test]
fn vobe_id_display_as_inner_string() {
    let id = VobeId::from_string("vobe_123");
    assert_eq!(id.to_string(), "vobe_123");
}

#[test]
fn vobe_id_from_str_rejects_empty() {
    let r: Result<VobeId, _> = "".parse();
    assert!(r.is_err());
}

#[test]
fn vobe_id_from_str_accepts_nonempty() {
    let id: VobeId = "vobe_xyz".parse().unwrap();
    assert_eq!(id.as_str(), "vobe_xyz");
}

#[test]
fn vobe_new_has_defaults() {
    let v = Vobe::new("demo", "/tmp/demo");
    assert_eq!(v.name, "demo");
    assert_eq!(v.path, std::path::PathBuf::from("/tmp/demo"));
    assert!(v.git.is_none());
    assert!(v.framework.is_none());
    assert!(v.language.is_none());
    assert!(v.package_manager.is_none());
    assert!(v.last_opened.is_none());
    assert!(v.last_modified.is_none());
    assert!(v.notes.is_none());
    assert!(v.tags.is_empty());
    assert!(!v.is_dirty());
    assert!(!v.has_unpushed());
    assert!(!v.has_unpulled());
    assert!(!v.is_archived());
}

#[test]
fn merge_detected_keeps_user_fields_and_reports_change() {
    let mut stored = Vobe::new("my-name", "/tmp/demo");
    stored.add_tag("work");
    stored.notes = Some("keep me".into());
    stored.pinned = true;
    stored.language = Some("Rust".into());
    stored.touch_opened();
    let opened = stored.last_opened;

    // Same detection -> unchanged.
    let mut detected = Vobe::new("detected-name", "/tmp/demo");
    detected.language = Some("Rust".into());
    let (merged, changed) = stored.merge_detected(&detected);
    assert!(!changed, "same detection must not count as changed");
    assert_eq!(merged.name, "my-name", "the user's name wins");
    assert_eq!(merged.tags, vec!["work".to_string()]);
    assert_eq!(merged.notes.as_deref(), Some("keep me"));
    assert!(merged.pinned);
    assert_eq!(merged.last_opened, opened);
    assert_eq!(merged.id, stored.id);
    assert_eq!(merged.language.as_deref(), Some("Rust"));

    // Changed stack -> reported as changed.
    let mut detected2 = Vobe::new("detected-name", "/tmp/demo");
    detected2.language = Some("TypeScript".into());
    let (merged2, changed2) = stored.merge_detected(&detected2);
    assert!(changed2, "a different language must count as changed");
    assert_eq!(merged2.language.as_deref(), Some("TypeScript"));
    assert_eq!(merged2.name, "my-name");

    // Changed git state -> reported as changed.
    let mut detected3 = Vobe::new("detected-name", "/tmp/demo");
    detected3.git = Some(GitInfo::clean("main"));
    let (_, changed3) = stored.merge_detected(&detected3);
    assert!(changed3, "a different git state must count as changed");
}

#[test]
fn detection_differs_ignores_user_fields_and_timestamps() {
    let mut a = Vobe::new("a", "/tmp/a");
    a.touch_modified();
    let mut b = Vobe::new("b", "/tmp/b");
    b.touch_modified();
    b.add_tag("other");
    b.notes = Some("different".into());
    b.pinned = true;
    // The name, path, tags, notes, pin and timestamps are the user's or
    // the clock's — none of them mean the project changed.
    assert!(!a.detection_differs(&b));
}

#[test]
fn vobe_tags_add_remove_query() {
    let mut v = Vobe::new("demo", "/tmp/demo");
    assert!(v.add_tag("work"));
    assert!(!v.add_tag("work")); // dup
    assert!(v.has_tag("work"));
    assert!(v.add_tag("archived"));
    assert!(v.is_archived());
    assert!(v.remove_tag("archived"));
    assert!(!v.is_archived());
    assert!(!v.remove_tag("nope"));
}

#[test]
fn vobe_touch_helpers_set_timestamps() {
    let mut v = Vobe::new("demo", "/tmp/demo");
    let before = Utc::now();
    v.touch_opened();
    v.touch_modified();
    assert!(v.last_opened.unwrap() >= before);
    assert!(v.last_modified.unwrap() >= before);
}

#[test]
fn vobe_serde_roundtrip_preserves_fields() {
    let mut v = Vobe::new("demo", "/tmp/demo");
    v.language = Some("TypeScript".into());
    v.framework = Some("Next.js".into());
    v.package_manager = Some("pnpm".into());
    let s = serde_json::to_string(&v).unwrap();
    let back: Vobe = serde_json::from_str(&s).unwrap();
    assert_eq!(v, back);
    // skip_serializing_if working
    let obj: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(obj.get("notes").is_none());
    assert!(obj.get("last_opened").is_none());
}

#[test]
fn git_info_helpers() {
    let dirty = GitInfo {
        branch: "main".into(),
        dirty: true,
        ahead: 0,
        behind: 0,
        last_commit: None,
    };
    assert!(dirty.is_dirty());
    assert!(!dirty.is_clean());
    assert!(!dirty.needs_push());
    assert!(!dirty.needs_pull());

    let push_pull = GitInfo {
        branch: "main".into(),
        dirty: false,
        ahead: 2,
        behind: 1,
        last_commit: None,
    };
    assert!(push_pull.needs_push());
    assert!(push_pull.needs_pull());
    assert!(!push_pull.is_clean());

    let clean = GitInfo::clean("main");
    assert!(clean.is_clean());
    assert_eq!(clean.branch, "main");
}

#[test]
fn commit_short_hash_and_message() {
    let c = Commit {
        hash: "abcdef1234567890".into(),
        message: "fix: handle null pointer in parser".into(),
        author: "Yizhou Jiang".into(),
        date: Utc::now(),
    };
    assert_eq!(c.short_hash(), "abcdef1");
    let short_msg = c.short_message(10);
    assert!(short_msg.ends_with('…'));
    // 10 visible chars + 1 ellipsis
    assert_eq!(short_msg.chars().count(), 11);
    let fits = c.short_message(100);
    assert_eq!(fits, c.message);
}

#[test]
fn activity_kind_labels() {
    assert_eq!(ActivityKind::Opened.label(), "opened");
    assert_eq!(ActivityKind::Modified.to_string(), "modified");
    assert_eq!(ActivityKind::Committed.to_string(), "committed");
    assert_eq!(ActivityKind::Created.label(), "created");
}

#[test]
fn activity_kind_serde_roundtrip() {
    let kinds = [
        ActivityKind::Opened,
        ActivityKind::Modified,
        ActivityKind::Committed,
        ActivityKind::Scanned,
        ActivityKind::Created,
        ActivityKind::Tagged,
        ActivityKind::Noted,
    ];
    for k in kinds {
        let s = serde_json::to_string(&k).unwrap();
        let back: ActivityKind = serde_json::from_str(&s).unwrap();
        assert_eq!(k, back);
    }
}

#[test]
fn activity_event_builder() {
    let id = VobeId::from_string("vobe_x");
    let ev = ActivityEvent::now(id.clone(), ActivityKind::Opened).with_detail("via vbs");
    assert_eq!(ev.vobe_id, id);
    assert_eq!(ev.kind, ActivityKind::Opened);
    assert_eq!(ev.detail.as_deref(), Some("via vbs"));
    assert!(ev.id.is_none());
}
