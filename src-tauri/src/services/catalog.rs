//! The question catalog — a first-class, shipped part of the app, loaded at
//! startup exactly like the other bundled resources (`presets/`,
//! `test-packs.json.gz`). Each question is merged into a `Problem` via the
//! `lc_import` engine (a matched + fingerprint-verified pack becomes the hidden
//! judge). There is no app-data copy, no runtime import, and no fallback: the
//! catalog is application content, not something the user supplies.
//!
//! Name-agnostic by design: a "catalog" is ANY file in `resources/` named
//! `catalog*.json` or `catalog*.json.gz`. Drop in `catalog.json`,
//! `catalog_leetcode.json`, `catalog_ourown.json.gz`, … and it is discovered
//! and loaded — no code change, no hardcoded filename. Multiple catalogs merge
//! (de-duplicated by slug), so an original catalog can be added alongside or
//! swapped in for another. This keeps authoring frictionless: to bring your own
//! library you only add files, you never edit the loader.
//!
//! Legal: whatever is *committed/shipped* here MUST be original, shippable
//! content — never a third party's problem text (`.docs/CONTENT_MAP.md`). The
//! LeetCode scrape is a local dev input only and is gitignored (`*leetcode*`).

use std::collections::{BTreeMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::domain::problem::Problem;
use crate::error::AppResult;
use crate::services::{lc_import, pack_store::PackStore, preset_store::PresetStore};

/// Every catalog file bundled in `resources_dir`, in deterministic order.
///
/// A catalog is any file whose name starts with `catalog` and ends in `.json`
/// or `.json.gz` — the loader does not care what it is called, so a dev scrape
/// (`catalog_leetcode.json`) and a shipped original catalog (`catalog.json.gz`)
/// can sit side by side and either can be swapped without touching code. For a
/// given stem the gzipped form wins over the plain one; results are sorted by
/// name so the merged problem order is stable. Empty ⇒ no catalog is bundled
/// (the library is empty until one is added).
pub fn resource_paths(resources_dir: &Path) -> Vec<PathBuf> {
    let mut by_stem: BTreeMap<String, PathBuf> = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(resources_dir) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let (stem, is_gz) = match (name.strip_suffix(".json.gz"), name.strip_suffix(".json")) {
            (Some(stem), _) => (stem, true),
            (None, Some(stem)) => (stem, false),
            _ => continue,
        };
        if !stem.starts_with("catalog") {
            continue;
        }
        // Insert on first sight; otherwise let a gzipped file displace a plain one.
        let existing_is_gz = by_stem
            .get(stem)
            .is_some_and(|p| p.extension().is_some_and(|e| e == "gz"));
        if !by_stem.contains_key(stem) || (is_gz && !existing_is_gz) {
            by_stem.insert(stem.to_string(), path);
        }
    }
    by_stem.into_values().collect()
}

/// Reads the catalog file (gzip-decoding when the path ends in `.gz`) and merges
/// every question into a `Problem`. No DB writes, no stress materialization —
/// browse/run uses visible examples + pack tests.
pub fn load(packs: &PackStore, presets: &PresetStore, path: &Path) -> AppResult<Vec<Problem>> {
    let json = read_catalog(path)?;
    let scrape = lc_import::parse_scrape(&json)?;
    let presets = presets.all();
    let mut out = Vec::new();
    let mut number = 1u32;
    for q in &scrape.questions {
        let no_statement = q.body_text.trim().is_empty() && q.body_html.trim().is_empty();
        if no_statement || q.code_stubs.python.trim().is_empty() {
            continue;
        }
        let pack = packs.get(&q.slug);
        let verified =
            pack.filter(|p| lc_import::verify_fingerprint(&p.entry_point, &q.code_stubs.python));
        let merged = lc_import::merge_question(q, verified, vec![], presets, number);
        out.push(merged.problem);
        number += 1;
    }
    Ok(out)
}

/// Discovers every catalog in `resources_dir`, loads each, and merges them into
/// one problem list. Questions are de-duplicated by slug (the first catalog to
/// define a slug wins), and the survivors are renumbered `1..=N` in load order
/// so display numbers stay contiguous no matter how many catalogs contributed.
/// This is the entry point the app uses at startup.
pub fn load_all(
    packs: &PackStore,
    presets: &PresetStore,
    resources_dir: &Path,
) -> AppResult<Vec<Problem>> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<Problem> = Vec::new();
    for path in resource_paths(resources_dir) {
        for problem in load(packs, presets, &path)? {
            if seen.insert(problem.id.clone()) {
                out.push(problem);
            }
        }
    }
    for (i, problem) in out.iter_mut().enumerate() {
        problem.number = (i as u32) + 1;
    }
    Ok(out)
}

/// Reads the catalog to a string, transparently gunzipping `*.gz`.
fn read_catalog(path: &Path) -> AppResult<String> {
    if path.extension().is_some_and(|ext| ext == "gz") {
        let file = std::fs::File::open(path)?;
        let mut decoder = flate2::read::GzDecoder::new(file);
        let mut json = String::new();
        decoder.read_to_string(&mut json)?;
        Ok(json)
    } else {
        Ok(std::fs::read_to_string(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("anvil-catalog-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The real bundled resources dir (packs + presets live here, and locally
    /// so does the gitignored `catalog_leetcode.json`).
    fn real_resources() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("resources")
    }

    fn real_packs() -> PackStore {
        PackStore::new(real_resources().join("test-packs.json.gz"))
    }

    fn real_presets() -> PresetStore {
        PresetStore::load(&real_resources().join("presets")).unwrap()
    }

    /// One catalog question whose python stub matches `method`, so the frozen
    /// pack for `slug` fingerprint-verifies.
    fn question_json(slug: &str, method: &str) -> String {
        format!(
            r#"{{"qid":"1","slug":"{slug}","title":"{slug}","difficulty":"Easy",
            "body_text":"do it","body_html":"<p>do it</p>",
            "code_stubs":{{"python":"class Solution:\n    def {method}(self, nums, target):\n        pass\n","javascript":"x"}},
            "example_tests":[]}}"#
        )
    }

    /// Wraps question objects into a catalog file body.
    fn catalog_json(questions: &[String]) -> String {
        format!(
            r#"{{"schema_version":"1.0","questions":[{}]}}"#,
            questions.join(",")
        )
    }

    #[test]
    fn resource_paths_discovers_by_convention_and_prefers_gzip() {
        let dir = scratch("resolve");
        assert!(resource_paths(&dir).is_empty());

        std::fs::write(dir.join("catalog.json"), b"{}").unwrap();
        std::fs::write(dir.join("catalog_leetcode.json"), b"{}").unwrap();
        std::fs::write(dir.join("notes.json"), b"{}").unwrap(); // wrong prefix → ignored
        let paths = resource_paths(&dir);
        assert_eq!(paths.len(), 2, "both catalog* files, never notes.json");
        assert!(paths
            .iter()
            .all(|p| p.file_name().unwrap().to_string_lossy().starts_with("catalog")));

        // For the same stem the gzipped form wins.
        std::fs::write(dir.join("catalog.json.gz"), b"gz").unwrap();
        let catalog = resource_paths(&dir)
            .into_iter()
            .find(|p| p.file_name().unwrap().to_string_lossy().starts_with("catalog."))
            .unwrap();
        assert!(catalog.to_string_lossy().ends_with("catalog.json.gz"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn maps_any_catalog_name_to_the_frozen_packs() {
        // A catalog with a deliberately non-standard name still maps to the
        // frozen two-sum pack — the hallmark of the name-agnostic design.
        let dir = scratch("anyname");
        std::fs::write(
            dir.join("catalog_totally_custom_name.json"),
            catalog_json(&[question_json("two-sum", "twoSum")]),
        )
        .unwrap();

        let problems = load_all(&real_packs(), &real_presets(), &dir).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].id, "two-sum");
        assert!(
            problems[0].judge.is_some(),
            "two-sum must map to its verified pack → hidden judge"
        );
        assert!(problems[0].entry_point.is_some());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn merges_multiple_catalogs_and_dedups_by_slug() {
        // Two catalogs; `two-sum` appears in both, `3sum` only in the second.
        let dir = scratch("multi");
        std::fs::write(
            dir.join("catalog.json"),
            catalog_json(&[question_json("two-sum", "twoSum")]),
        )
        .unwrap();
        std::fs::write(
            dir.join("catalog_extra.json"),
            catalog_json(&[
                question_json("two-sum", "twoSum"),
                question_json("3sum", "threeSum"),
            ]),
        )
        .unwrap();

        let problems = load_all(&real_packs(), &real_presets(), &dir).unwrap();
        let slugs: Vec<&str> = problems.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(slugs, vec!["two-sum", "3sum"], "deduped; catalog.json wins");
        assert_eq!(problems[0].number, 1, "renumbered contiguously across files");
        assert_eq!(problems[1].number, 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// End-to-end against the real (gitignored) LeetCode catalog when present:
    /// proves the full file loads and most questions map to the frozen packs.
    /// Skipped in environments without the dev scrape (e.g. CI).
    #[test]
    fn real_catalog_maps_to_frozen_packs() {
        let res = real_resources();
        if resource_paths(&res).is_empty() {
            eprintln!("SKIPPED: no catalog*.json in resources/ (dev scrape absent)");
            return;
        }
        let problems = load_all(&real_packs(), &real_presets(), &res).unwrap();
        let verified = problems.iter().filter(|p| p.judge.is_some()).count();
        eprintln!(
            "real catalog: {} problems, {} mapped to frozen packs",
            problems.len(),
            verified
        );
        assert!(problems.len() > 1000, "expected a full catalog, got {}", problems.len());
        assert!(verified > 2000, "most problems should map to packs, got {verified}");
    }

    #[test]
    fn read_catalog_handles_gzip_and_plain() {
        let dir = scratch("read");
        let body = br#"{"schema_version":"3","questions":[{"slug":"two-sum"}]}"#;

        let plain = dir.join("catalog.json");
        std::fs::write(&plain, body).unwrap();
        assert!(read_catalog(&plain).unwrap().contains("two-sum"));

        let gz = dir.join("catalog.json.gz");
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(body).unwrap();
        std::fs::write(&gz, enc.finish().unwrap()).unwrap();
        assert!(read_catalog(&gz).unwrap().contains("two-sum"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
