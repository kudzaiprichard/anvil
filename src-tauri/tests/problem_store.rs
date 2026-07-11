//! Loader tests (task 0002): happy path against fixtures, three failure
//! paths with useful messages, and a guard that the real bundled bank is
//! valid so a bad resource file can never reach a release.

use std::path::PathBuf;

use app_lib::services::problem_store::ProblemStore;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn loads_valid_fixture_bank() {
    let store = ProblemStore::load(&fixture("problems-valid")).expect("should load");
    assert_eq!(store.all().len(), 3);
    // sorted by number regardless of directory walk order
    let numbers: Vec<u32> = store.all().iter().map(|p| p.number).collect();
    assert_eq!(numbers, vec![1, 2, 3]);
    assert!(store.get("fixture-echo-text").is_some());
    assert!(store.get("nope").is_none());
}

#[test]
fn bad_json_fails_naming_the_file() {
    let err = ProblemStore::load(&fixture("problems-bad-json")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("fixture-corrupt.json"), "message was: {msg}");
}

#[test]
fn duplicate_id_is_rejected() {
    let err = ProblemStore::load(&fixture("problems-dup-id")).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("duplicate problem id 'fixture-twin'"),
        "message was: {msg}"
    );
}

#[test]
fn missing_hidden_test_is_rejected() {
    let err = ProblemStore::load(&fixture("problems-missing-hidden")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("fixture-no-hidden.json"), "message was: {msg}");
    assert!(msg.contains("hidden"), "message was: {msg}");
}

#[test]
fn missing_dir_is_rejected() {
    let err = ProblemStore::load(&fixture("does-not-exist")).unwrap_err();
    assert!(err.to_string().contains("does-not-exist"));
}

// (Removed: the shipped built-in bank no longer exists — the catalog loads
// from the local scrape. The fixture-based loader tests above still cover
// ProblemStore::load for when built-ins are re-added.)
