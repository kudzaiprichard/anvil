//! Stress-materialization integration tests (task 0008):
//! a pack's deterministic generator specs become literal hidden test
//! cases through the real sandbox, so the runtime runner's `cases.json`
//! contract never changes. Runtime-gated like all sandbox tests.

mod common;

use std::path::PathBuf;

use app_lib::services::pack_store::{materialize_stress, PackStore};

fn shipped_store() -> PackStore {
    let bundle = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("test-packs.json.gz");
    PackStore::new(bundle)
}

#[test]
fn stress_materialization_is_deterministic_and_sandbox_computed() {
    require_runtime!("python");
    let store = shipped_store();
    let pack = store.get("two-sum").expect("fixture pack present");
    assert!(
        !pack.stress.is_empty(),
        "fixture must exercise the stress path"
    );

    let first = materialize_stress(pack, "python");
    let second = materialize_stress(pack, "python");

    // A CI sandbox that caps threads can't launch the harness ("runner error:
    // ... can't start new thread"), skipping every spec. That's an environment
    // limit, not a determinism/product bug — skip. A skip from any other cause
    // still trips the assertion below.
    if common::stress_skipped_by_sandbox(&first.skipped) {
        eprintln!(
            "SKIPPED: sandbox could not execute stress generators here: {:?}",
            first.skipped
        );
        return;
    }

    // Every spec materialized (no skips) and identical across runs — the
    // seeded `gen(rng, size)` contract plus a deterministic reference
    // solution must reproduce byte-identical cases.
    assert!(
        first.skipped.is_empty(),
        "unexpected skips: {:?}",
        first.skipped
    );
    assert_eq!(first.cases.len(), pack.stress.len());
    assert_eq!(first, second, "materialization is not deterministic");

    // The expected value is computed by execution, not shipped as a literal:
    // the args list is large (10k elements) and the answer is a real index
    // pair the reference solution produced.
    let case = &first.cases[0];
    assert!(case.hidden, "stress cases are hidden judge cases");
    let nums = case.input[0].as_array().expect("first arg is the array");
    assert!(nums.len() >= 1000, "stress input should be large");
    let expected = case.expected.as_array().expect("two-sum returns a pair");
    assert_eq!(expected.len(), 2);
}

#[test]
fn a_failing_generator_degrades_per_case_without_panicking() {
    require_runtime!("python");
    let store = shipped_store();
    let mut pack = store.get("two-sum").expect("fixture pack present").clone();
    // Corrupt the generator so it raises at runtime; the whole question must
    // not blow up — the spec is skipped with a recorded reason.
    pack.stress[0].generator_python = "def gen(rng, size):\n    raise ValueError('boom')\n".into();
    let out = materialize_stress(&pack, "python");
    assert!(out.cases.is_empty());
    assert_eq!(out.skipped.len(), 1);
    assert!(out.skipped[0].contains("skipped"), "{:?}", out.skipped);
}
