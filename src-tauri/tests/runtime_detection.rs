//! Runtime detection integration tests (task 0007). Detection must never
//! panic regardless of machine state; with an emptied PATH everything is
//! `found: false`. The PATH mutation is process-global, so these tests
//! share a lock.

use app_lib::services::runtime_detect::{detect, RuntimeInfo};

static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn shape_is_valid(rt: &RuntimeInfo) {
    assert!(
        rt.tag == "Py" || rt.tag == "JS",
        "unexpected tag {}",
        rt.tag
    );
    if rt.found {
        assert!(!rt.path.is_empty(), "{}: found but path empty", rt.name);
        assert!(
            rt.version.starts_with('v'),
            "{}: version {}",
            rt.name,
            rt.version
        );
    } else {
        assert!(rt.path.is_empty());
        assert!(rt.version.is_empty());
    }
}

#[test]
fn detection_never_panics_and_reports_both_runtimes() {
    let _serial = serial();
    let detected = detect();
    assert_eq!(detected.len(), 2);
    for rt in &detected {
        shape_is_valid(rt);
    }
}

#[test]
fn empty_path_yields_all_not_found() {
    let _serial = serial();
    let saved = std::env::var_os("PATH");
    std::env::set_var("PATH", "");
    let detected = detect();
    if let Some(saved) = saved {
        std::env::set_var("PATH", saved);
    }
    assert!(detected.iter().all(|rt| !rt.found), "{detected:?}");
}
