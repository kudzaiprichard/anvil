//! Test-pack bundle access (task 0008): opens the
//! shipped `test-packs.json.gz` lazily on first use (startup is untouched),
//! indexes packs by slug, and materializes stress generator specs into
//! literal hidden test cases through the existing sandbox — so the runtime
//! runner's `cases.json` contract never changes.

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::domain::pack::{StressSpec, TestPack};
use crate::domain::problem::{Judge, TestCase};
use crate::domain::run::Language;
use crate::error::{AppError, AppResult};
use crate::services::runner;

pub struct PackStore {
    bundle_path: PathBuf,
    packs: OnceLock<HashMap<String, TestPack>>,
}

impl PackStore {
    /// No IO — the bundle is opened on first `get()`/`coverage()`.
    pub fn new(bundle_path: PathBuf) -> Self {
        Self {
            bundle_path,
            packs: OnceLock::new(),
        }
    }

    /// Parse-once lazy map. A missing or corrupt bundle degrades to an
    /// empty store (every import falls back to basic mode — fail closed)
    /// rather than failing the app.
    fn packs(&self) -> &HashMap<String, TestPack> {
        self.packs
            .get_or_init(|| match load_bundle(&self.bundle_path) {
                Ok(packs) => {
                    log::info!("loaded {} test packs", packs.len());
                    packs
                }
                Err(e) => {
                    log::error!(
                        "test-pack bundle unusable ({}): {e} — imports get basic mode",
                        self.bundle_path.display()
                    );
                    HashMap::new()
                }
            })
    }

    pub fn get(&self, slug: &str) -> Option<&TestPack> {
        self.packs().get(slug)
    }

    /// Number of shipped packs (the import preview's "N of your questions
    /// have packs" denominator).
    pub fn coverage(&self) -> usize {
        self.packs().len()
    }
}

fn load_bundle(path: &std::path::Path) -> AppResult<HashMap<String, TestPack>> {
    let file = std::fs::File::open(path)?;
    let mut decoder = flate2::read::GzDecoder::new(file);
    let mut json = String::new();
    decoder.read_to_string(&mut json)?;
    let packs: HashMap<String, TestPack> = serde_json::from_str(&json)
        .map_err(|e| AppError::Validation(format!("invalid pack bundle: {e}")))?;
    // Quarantined packs must never reach the bundle; treat any that slipped
    // through as data errors and drop them.
    let (good, bad): (HashMap<_, _>, HashMap<_, _>) =
        packs.into_iter().partition(|(_, p)| p.verified);
    for slug in bad.keys() {
        log::error!("pack '{slug}' is unverified — dropped from the store");
    }
    Ok(good)
}

/// Outcome of materializing one pack's stress specs: literal hidden cases
/// plus per-spec skip reasons (a failing generator or solution never blocks
/// the whole question).
#[derive(Debug, Default, PartialEq)]
pub struct MaterializedStress {
    pub cases: Vec<TestCase>,
    pub skipped: Vec<String>,
}

/// Runs each stress generator (our pack-shipped code) in the sandbox to
/// produce input args, then the pack's Python reference solution on those
/// args to compute the expected value — both deterministic (seeded RNG
/// contract: `gen(rng, size)`). `python_program` is the detected
/// interpreter path.
pub fn materialize_stress(pack: &TestPack, python_program: &str) -> MaterializedStress {
    let mut out = MaterializedStress::default();
    for spec in &pack.stress {
        match materialize_one(pack, spec, python_program) {
            Ok(case) => out.cases.push(case),
            Err(e) => out
                .skipped
                .push(format!("stress '{}' skipped: {e}", spec.description)),
        }
    }
    out
}

fn materialize_one(
    pack: &TestPack,
    spec: &StressSpec,
    python_program: &str,
) -> AppResult<TestCase> {
    // The generator runs through the same harness as everything else, via a
    // wrapper entry point that seeds the RNG from the spec.
    let generator_code = format!(
        "import random\n{}\n\ndef __anvil_gen(seed, size):\n    rng = random.Random(seed)\n    return list(gen(rng, size))\n",
        spec.generator_python
    );
    // The generator returns the raw args array (plain JSON) — never node types,
    // so io_types is None here.
    let args_value = runner::compute_outputs(
        Language::Python,
        &generator_code,
        Some("__anvil_gen"),
        &Judge::Exact,
        &[vec![
            serde_json::json!(spec.seed),
            serde_json::json!(spec.size),
        ]],
        python_program,
        None,
    )?
    .pop()
    .ok_or_else(|| AppError::Runner("generator produced no output".into()))?;
    let args = args_value
        .as_array()
        .cloned()
        .ok_or_else(|| AppError::Runner("generator must return an args list".into()))?;

    // Expected value = the verified reference solution executed on those
    // args, honoring the pack's judge semantics (in_place packs store the
    // mutated argument). For node packs the pack's io_types drives the
    // array⇄ListNode/TreeNode (de)serialization at the harness boundary.
    let expected = runner::compute_outputs(
        Language::Python,
        &pack.solutions.python,
        Some(pack.entry_point.python.as_str()),
        &pack.judge,
        std::slice::from_ref(&args),
        python_program,
        pack.entry_point.io_types.as_ref(),
    )?
    .pop()
    .ok_or_else(|| AppError::Runner("reference solution produced no output".into()))?;

    Ok(TestCase {
        input: args,
        expected,
        hidden: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn shipped_bundle() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test-packs.json.gz")
    }

    #[test]
    fn fixture_bundle_loads_lazily_and_indexes_by_slug() {
        let store = PackStore::new(shipped_bundle());
        let pack = store.get("two-sum").expect("fixture pack present");
        assert_eq!(pack.qid, "1");
        assert_eq!(pack.entry_point.python, "Solution.twoSum");
        assert!(pack.verified);
        assert!(pack.tests.len() >= 3);
        // Growth-robust: the bundle starts at the batch-0 floor and only ever
        // grows as batches are authored (two-sum is frozen, so it never drops).
        assert!(store.coverage() >= 5);
        // unknown slug is a cheap miss
        assert!(store.get("no-such-slug").is_none());
    }

    #[test]
    fn missing_bundle_degrades_to_an_empty_store() {
        let store = PackStore::new(PathBuf::from("Z:/nowhere/test-packs.json.gz"));
        assert!(store.get("two-sum").is_none());
        assert_eq!(store.coverage(), 0);
    }

    #[test]
    fn unverified_packs_are_dropped_as_data_errors() {
        let mut packs: HashMap<String, crate::domain::pack::TestPack> = {
            let file = std::fs::File::open(shipped_bundle()).unwrap();
            let mut decoder = flate2::read::GzDecoder::new(file);
            let mut json = String::new();
            std::io::Read::read_to_string(&mut decoder, &mut json).unwrap();
            serde_json::from_str(&json).unwrap()
        };
        for pack in packs.values_mut() {
            pack.verified = false;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json.gz");
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(
            &mut encoder,
            serde_json::to_string(&packs).unwrap().as_bytes(),
        )
        .unwrap();
        std::fs::write(&path, encoder.finish().unwrap()).unwrap();

        let store = PackStore::new(path);
        assert_eq!(store.coverage(), 0);
    }
}
