//! Catalog stability smoke test: load the real bundled catalog exactly as the
//! app does at startup, sample N problems, and drive each one's shipped
//! reference solution through the real `runner::execute` against its hidden
//! judge — in both Python and JavaScript. A green run means the catalog loads,
//! maps to the frozen packs, and those problems genuinely run & judge.
//!
//! Runtime-gated (skips without python/node) and reproducible: the sample is a
//! seeded shuffle, override with `ANVIL_SMOKE_SEED` / `ANVIL_SMOKE_N`.

mod common;

use app_lib::domain::run::{Language, RunStatus};
use app_lib::services::preset_store::PresetStore;
use app_lib::services::{catalog, pack_store::PackStore, runner};
use std::path::PathBuf;

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
}

fn env_num(key: &str, default: u64) -> u64 {
    let Ok(raw) = std::env::var(key) else {
        return default;
    };
    let v = raw.trim();
    let parsed = match v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16),
        None => v.parse(),
    };
    parsed.unwrap_or_else(|_| panic!("{key}={raw:?} is not a valid number"))
}

/// Deterministic index shuffle (splitmix64) so "random 5" is reproducible but
/// varies with the seed.
fn sample_indices(len: usize, n: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..len).collect();
    let mut state = seed;
    // Fisher–Yates with a splitmix64 stream.
    for i in (1..len).rev() {
        state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        let j = (z % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    order.into_iter().take(n).collect()
}

#[test]
fn sampled_catalog_problems_run_and_judge_green() {
    require_runtime!("python");

    let packs = PackStore::new(resources().join("test-packs.json.gz"));
    let presets = PresetStore::load(&resources().join("presets")).unwrap();

    let problems = catalog::load_all(&packs, &presets, &resources()).unwrap();
    if problems.is_empty() {
        eprintln!("SKIPPED: no catalog*.json in resources/ (dev scrape absent)");
        return;
    }

    // Only full-tier problems: a hidden judge AND a shipped reference solution.
    let candidates: Vec<_> = problems
        .iter()
        .filter(|p| {
            p.judge.is_some()
                && p.reference_solution
                    .as_ref()
                    .and_then(|r| r.python.as_ref())
                    .is_some_and(|s| !s.trim().is_empty())
        })
        .collect();
    assert!(
        candidates.len() > 100,
        "expected a well-populated catalog, got {} full-tier problems",
        candidates.len()
    );

    let n = env_num("ANVIL_SMOKE_N", 5) as usize;
    let seed = env_num("ANVIL_SMOKE_SEED", 0xA5A5_1234_DEAD_BEEF);
    let picks = sample_indices(candidates.len(), n.min(candidates.len()), seed);
    let node = common::runtime_available("node");

    eprintln!(
        "\n=== catalog smoke: {} full-tier problems, sampling {} (seed {:#x}, node={}) ===",
        candidates.len(),
        picks.len(),
        seed,
        node
    );

    let mut failures: Vec<String> = Vec::new();

    for idx in picks {
        let p = candidates[idx];
        let hidden = p.test_cases.iter().filter(|c| c.hidden).count();
        eprintln!(
            "\n#{:<5} {:<40} [{:?}]  {} hidden cases",
            p.number, p.id, p.difficulty, hidden
        );

        // Python reference must pass every case.
        let py = p
            .reference_solution
            .as_ref()
            .unwrap()
            .python
            .clone()
            .unwrap();
        match runner::execute(p, Language::Python, &py, true) {
            Ok(r) if r.status == RunStatus::Pass && r.passed == r.total => {
                eprintln!("   python: PASS {}/{}", r.passed, r.total);
            }
            Ok(r) => {
                let bad = r
                    .cases
                    .iter()
                    .find(|c| !c.passed)
                    .map(|c| format!("case #{}", c.index))
                    .unwrap_or_default();
                let msg = format!(
                    "{} python {:?} {}/{} {} err={:?}",
                    p.id, r.status, r.passed, r.total, bad, r.error
                );
                eprintln!("   python: FAIL — {msg}");
                failures.push(msg);
            }
            Err(e) => {
                let msg = format!("{} python execute error: {e}", p.id);
                eprintln!("   python: ERROR — {e}");
                failures.push(msg);
            }
        }

        // JavaScript reference (when node is present and one ships).
        if node {
            if let Some(js) = p
                .reference_solution
                .as_ref()
                .and_then(|r| r.javascript.clone())
            {
                if !js.trim().is_empty() {
                    match runner::execute(p, Language::Javascript, &js, true) {
                        Ok(r) if r.status == RunStatus::Pass && r.passed == r.total => {
                            eprintln!("   node:   PASS {}/{}", r.passed, r.total);
                        }
                        Ok(r) => {
                            let msg = format!(
                                "{} js {:?} {}/{} err={:?}",
                                p.id, r.status, r.passed, r.total, r.error
                            );
                            eprintln!("   node:   FAIL — {msg}");
                            failures.push(msg);
                        }
                        Err(e) => {
                            let msg = format!("{} js execute error: {e}", p.id);
                            eprintln!("   node:   ERROR — {e}");
                            failures.push(msg);
                        }
                    }
                }
            }
        }
    }

    eprintln!("\n=== done: {} failure(s) ===\n", failures.len());
    assert!(
        failures.is_empty(),
        "reference solutions failed:\n{}",
        failures.join("\n")
    );
}
