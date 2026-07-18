//! Round-trip tests for the SQLite store.

use std::path::PathBuf;
use vobes_core::{ActivityEvent, ActivityKind, GitInfo, Vobe};
use vobes_store::{Filter, Sort, SqliteStore, Store};

fn sample_vobe(name: &str) -> Vobe {
    let mut v = Vobe::new(name, PathBuf::from(format!("/tmp/{name}")));
    v.framework = Some("Tauri".into());
    v.language = Some("Rust".into());
    v.package_manager = Some("cargo".into());
    v.add_tag("work");
    v.notes = Some("hello".into());
    v.touch_opened();
    v.touch_modified();
    v
}

#[test]
fn upsert_and_fetch_by_id_name_path() {
    let store = SqliteStore::open_in_memory().unwrap();
    let v = sample_vobe("alpha");
    let id = v.id.clone();
    store.upsert_vobe(&v).unwrap();

    let by_id = store.get_vobe(&id).unwrap().unwrap();
    assert_eq!(by_id, v);
    let by_name = store.get_vobe_by_name("alpha").unwrap().unwrap();
    assert_eq!(by_name, v);
    let by_path = store
        .get_vobe_by_path(&PathBuf::from("/tmp/alpha"))
        .unwrap()
        .unwrap();
    assert_eq!(by_path, v);
}

#[test]
fn upsert_updates_existing() {
    let store = SqliteStore::open_in_memory().unwrap();
    let mut v = sample_vobe("alpha");
    store.upsert_vobe(&v).unwrap();
    v.notes = Some("updated".into());
    store.upsert_vobe(&v).unwrap();
    let fetched = store.get_vobe_by_name("alpha").unwrap().unwrap();
    assert_eq!(fetched.notes.as_deref(), Some("updated"));
}

#[test]
fn list_vobes_sort_by_name() {
    let store = SqliteStore::open_in_memory().unwrap();
    for n in ["gamma", "alpha", "beta"] {
        store.upsert_vobe(&sample_vobe(n)).unwrap();
    }
    let list = store.list_vobes(&Filter::all(), Sort::Name).unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].name, "alpha");
    assert_eq!(list[1].name, "beta");
    assert_eq!(list[2].name, "gamma");
}

#[test]
fn list_filters_by_tag() {
    let store = SqliteStore::open_in_memory().unwrap();
    let mut a = sample_vobe("a");
    a.add_tag("personal");
    store.upsert_vobe(&a).unwrap();
    let b = sample_vobe("b");
    store.upsert_vobe(&b).unwrap();

    let personal = store
        .list_vobes(&Filter::all().with_tag("personal"), Sort::Name)
        .unwrap();
    assert_eq!(personal.len(), 1);
    assert_eq!(personal[0].name, "a");

    let work = store
        .list_vobes(&Filter::all().with_tag("work"), Sort::Name)
        .unwrap();
    assert_eq!(work.len(), 2);
}

#[test]
fn list_filters_dirty() {
    let store = SqliteStore::open_in_memory().unwrap();
    let mut clean = sample_vobe("clean");
    clean.path = PathBuf::from("/tmp/clean");
    store.upsert_vobe(&clean).unwrap();

    let mut dirty = sample_vobe("dirty");
    dirty.path = PathBuf::from("/tmp/dirty");
    dirty.git = Some(GitInfo::clean("main"));
    dirty.git.as_mut().unwrap().dirty = true;
    store.upsert_vobe(&dirty).unwrap();

    let dirty_list = store
        .list_vobes(&Filter::all().only_dirty(), Sort::Name)
        .unwrap();
    assert_eq!(dirty_list.len(), 1);
    assert_eq!(dirty_list[0].name, "dirty");
}

#[test]
fn list_excludes_archived() {
    let store = SqliteStore::open_in_memory().unwrap();
    let mut a = sample_vobe("a");
    a.path = PathBuf::from("/tmp/a");
    a.add_tag("archived");
    store.upsert_vobe(&a).unwrap();
    let b = sample_vobe("b");
    store.upsert_vobe(&b).unwrap();

    let visible = store
        .list_vobes(&Filter::all().exclude_archived(), Sort::Name)
        .unwrap();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "b");
}

#[test]
fn delete_cascades_activity() {
    let store = SqliteStore::open_in_memory().unwrap();
    let v = sample_vobe("a");
    let id = v.id.clone();
    store.upsert_vobe(&v).unwrap();

    let ev = ActivityEvent::now(id.clone(), ActivityKind::Opened);
    store.record_activity(&ev).unwrap();
    assert_eq!(store.recent_activity(10).unwrap().len(), 1);

    store.delete_vobe(&id).unwrap();
    assert!(store.get_vobe(&id).unwrap().is_none());
    // Activity for this vobe cascades away.
    assert_eq!(store.vobe_activity(&id, 10).unwrap().len(), 0);
}

#[test]
fn activity_persists_and_queries() {
    let store = SqliteStore::open_in_memory().unwrap();
    let a = sample_vobe("a");
    let a_id = a.id.clone();
    let b = sample_vobe("b");
    let b_id = b.id.clone();
    store.upsert_vobe(&a).unwrap();
    store.upsert_vobe(&b).unwrap();

    store
        .record_activity(&ActivityEvent::now(a_id.clone(), ActivityKind::Created))
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    store
        .record_activity(&ActivityEvent::now(b_id.clone(), ActivityKind::Created))
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    store
        .record_activity(&ActivityEvent::now(a_id.clone(), ActivityKind::Opened).with_detail("cli"))
        .unwrap();

    let recent = store.recent_activity(10).unwrap();
    assert_eq!(recent.len(), 3);
    // newest first
    assert_eq!(recent[0].kind, ActivityKind::Opened);
    assert_eq!(recent[2].kind, ActivityKind::Created);

    let a_acts = store.vobe_activity(&a_id, 10).unwrap();
    assert_eq!(a_acts.len(), 2);
    assert!(a_acts.iter().all(|e| e.vobe_id == a_id));

    let b_acts = store.vobe_activity(&b_id, 10).unwrap();
    assert_eq!(b_acts.len(), 1);
}

#[test]
fn export_and_import_round_trip() {
    let dir = std::env::temp_dir().join(format!("vobes-store-export-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let src = SqliteStore::open_in_memory().unwrap();
    let v = sample_vobe("alpha");
    let id = v.id.clone();
    src.upsert_vobe(&v).unwrap();
    src.record_activity(&ActivityEvent::now(id, ActivityKind::Created))
        .unwrap();

    let export_path = dir.join("snap.json");
    src.export_json(&export_path).unwrap();
    assert!(export_path.exists());

    let dst = SqliteStore::open_in_memory().unwrap();
    dst.import_json(&export_path).unwrap();

    let vobes = dst.list_vobes(&Filter::all(), Sort::Name).unwrap();
    assert_eq!(vobes.len(), 1);
    assert_eq!(vobes[0].name, "alpha");
    assert_eq!(dst.recent_activity(10).unwrap().len(), 1);
}

#[test]
fn open_creates_db_file() {
    let dir = std::env::temp_dir().join(format!("vobes-store-file-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let p = dir.join("vobes.db");
    let _store = SqliteStore::open(&p).unwrap();
    assert!(p.exists(), "db file should be created");
}
