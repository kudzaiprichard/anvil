//! Always-on catalog source: the app's problem set is the LeetCode questions
//! in the user's local scrape (`.docs/my_questions.json`), converted to
//! `Problem` via the existing `lc_import` merge engine. No DB, no UI — the
//! interactive importer was removed; this is the single source until a proper
//! import is re-added. The scrape is gitignored and never bundled/shipped.

use std::path::{Path, PathBuf};

use crate::domain::problem::Problem;
use crate::error::AppResult;
use crate::services::{lc_import, pack_store::PackStore, preset_store::PresetStore};

/// `$ANVIL_SCRAPE` if it points at a file, else `.docs/my_questions.json` found
/// by walking up from the current dir (covers `tauri dev` running in
/// `src-tauri/`).
pub fn scrape_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ANVIL_SCRAPE") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let cand = dir.join(".docs").join("my_questions.json");
        if cand.is_file() {
            return Some(cand);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Parse the scrape and merge every question into a `Problem` (the pack becomes
/// the hidden judge when matched + fingerprint-verified). No DB writes, no
/// stress materialization — browse/run uses visible examples + pack tests.
pub fn load(packs: &PackStore, presets: &PresetStore, path: &Path) -> AppResult<Vec<Problem>> {
    let json = std::fs::read_to_string(path)?;
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
