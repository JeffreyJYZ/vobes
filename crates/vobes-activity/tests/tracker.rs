//! Tests for the in-memory tracker.

use vobes_activity::{InMemoryTracker, Tracker};
use vobes_core::{ActivityEvent, ActivityKind, VobeId};

fn ev(vobe_id: &VobeId, kind: ActivityKind) -> ActivityEvent {
    ActivityEvent::now(vobe_id.clone(), kind)
}

#[test]
fn record_and_recent_orders_newest_last() {
    let t = InMemoryTracker::new();
    let id = VobeId::from_string("vobe_a");
    t.record(&ev(&id, ActivityKind::Created)).unwrap();
    t.record(&ev(&id, ActivityKind::Opened)).unwrap();
    t.record(&ev(&id, ActivityKind::Modified)).unwrap();

    let recent = t.recent(10).unwrap();
    assert_eq!(recent.len(), 3);
    // recent() returns newest last (chronological order)
    assert_eq!(recent[0].kind, ActivityKind::Created);
    assert_eq!(recent[1].kind, ActivityKind::Opened);
    assert_eq!(recent[2].kind, ActivityKind::Modified);
}

#[test]
fn recent_respects_limit() {
    let t = InMemoryTracker::new();
    let id = VobeId::from_string("vobe_a");
    for _ in 0..5 {
        t.record(&ev(&id, ActivityKind::Modified)).unwrap();
    }
    assert_eq!(t.recent(3).unwrap().len(), 3);
    assert_eq!(t.recent(10).unwrap().len(), 5);
}

#[test]
fn for_vobe_isolates_events() {
    let t = InMemoryTracker::new();
    let a = VobeId::from_string("vobe_a");
    let b = VobeId::from_string("vobe_b");
    t.record(&ev(&a, ActivityKind::Created)).unwrap();
    t.record(&ev(&b, ActivityKind::Created)).unwrap();
    t.record(&ev(&a, ActivityKind::Opened)).unwrap();

    let only_a = t.for_vobe(&a, 10).unwrap();
    assert_eq!(only_a.len(), 2);
    assert!(only_a.iter().all(|e| e.vobe_id == a));

    let only_b = t.for_vobe(&b, 10).unwrap();
    assert_eq!(only_b.len(), 1);
    assert_eq!(only_b[0].vobe_id, b);
}

#[test]
fn empty_tracker_returns_empty() {
    let t = InMemoryTracker::new();
    assert!(t.recent(10).unwrap().is_empty());
    let id = VobeId::from_string("vobe_x");
    assert!(t.for_vobe(&id, 10).unwrap().is_empty());
}
