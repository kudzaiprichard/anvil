//! Problem file import/export (task 0011) — the pure half. The versioned
//! envelope is the future-proofing for multi-problem packs; reject unknown
//! majors with a clear error. Never trust an imported file: full structural
//! validation + id-collision handling before it touches the library.
//! Dialog/file IO lives in `commands/authoring.rs`.

use serde::{Deserialize, Serialize};

use crate::domain::problem::{Problem, ProblemSource};
use crate::error::{AppError, AppResult};

pub const ENVELOPE_VERSION: u32 = 1;
pub const PACK_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemEnvelope {
    pub anvil_problem: u32,
    pub exported_at: String,
    pub problem: Problem,
}

/// Multi-problem pack — the bulk channel (curated sets, user backups,
/// community packs). Same versioned-envelope future-proofing as the single
/// file; `name` is a human label shown on import.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemPack {
    pub anvil_pack: u32,
    pub exported_at: String,
    #[serde(default)]
    pub name: String,
    pub problems: Vec<Problem>,
}

/// Reassigns a problem for local storage: validate, suffix the id/title on
/// collision, stamp `source: imported`, and give it the next free number.
fn prepare_one(
    mut problem: Problem,
    id_exists: &impl Fn(&str) -> bool,
    next_number: u32,
) -> AppResult<Problem> {
    problem
        .validate_structure()
        .map_err(|msg| AppError::Validation(format!("Invalid problem '{}': {msg}", problem.id)))?;
    if id_exists(&problem.id) {
        let base = problem.id.clone();
        let mut n = 1;
        let mut candidate = format!("{base}-imported");
        while id_exists(&candidate) {
            n += 1;
            candidate = format!("{base}-imported-{n}");
        }
        problem.id = candidate;
        problem.title.push_str(" (imported)");
    }
    problem.source = ProblemSource::Imported;
    problem.number = next_number;
    Ok(problem)
}

/// Pretty-printed pack JSON for the export dialog.
pub fn export_pack(name: &str, problems: &[Problem], exported_at: &str) -> AppResult<String> {
    let pack = ProblemPack {
        anvil_pack: PACK_VERSION,
        exported_at: exported_at.to_string(),
        name: name.to_string(),
        problems: problems.to_vec(),
    };
    serde_json::to_string_pretty(&pack)
        .map_err(|e| AppError::Storage(format!("failed to encode problem pack: {e}")))
}

/// Detects whether a file is a single-problem envelope or a multi-problem
/// pack and parses accordingly, so one importer handles both. `id_exists`
/// reflects the live library; `first_number` is the next free number —
/// problems are assigned sequentially from there.
pub fn parse_any_import(
    json: &str,
    id_exists: impl Fn(&str) -> bool,
    first_number: u32,
) -> AppResult<Vec<Problem>> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|_| {
        AppError::Validation("This file is not a valid Anvil problem or pack file.".into())
    })?;

    if value.get("anvil_pack").is_some() {
        let pack: ProblemPack = serde_json::from_value(value).map_err(|_| {
            AppError::Validation("This file is not a valid Anvil problem pack.".into())
        })?;
        if pack.anvil_pack != PACK_VERSION {
            return Err(AppError::Validation(format!(
                "Unsupported pack version {} — this build of Anvil reads version {}.",
                pack.anvil_pack, PACK_VERSION
            )));
        }
        if pack.problems.is_empty() {
            return Err(AppError::Validation(
                "This pack contains no problems.".into(),
            ));
        }
        // reserve ids + numbers across the whole pack so members don't
        // collide with each other or the library
        let mut taken: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut out = Vec::with_capacity(pack.problems.len());
        let mut number = first_number;
        for problem in pack.problems {
            // `combined` is scoped to this block so its borrow of `taken`
            // is released before the insert below.
            let prepared = {
                let combined = |id: &str| id_exists(id) || taken.contains(id);
                prepare_one(problem, &combined, number)?
            };
            taken.insert(prepared.id.clone());
            number += 1;
            out.push(prepared);
        }
        Ok(out)
    } else {
        Ok(vec![parse_import(json, id_exists, first_number)?])
    }
}

/// Pretty-printed envelope JSON for the save dialog.
pub fn export_envelope(problem: &Problem, exported_at: &str) -> AppResult<String> {
    let envelope = ProblemEnvelope {
        anvil_problem: ENVELOPE_VERSION,
        exported_at: exported_at.to_string(),
        problem: problem.clone(),
    };
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| AppError::Storage(format!("failed to encode problem file: {e}")))
}

/// Parses + validates an imported file and prepares it for the local
/// library: `source: "imported"`, original author kept, fresh local
/// `number`, and an id/title suffix when the id is already taken.
pub fn parse_import(
    json: &str,
    id_exists: impl Fn(&str) -> bool,
    next_number: u32,
) -> AppResult<Problem> {
    let envelope: ProblemEnvelope = serde_json::from_str(json).map_err(|_| {
        AppError::Validation("This file is not a valid Anvil problem export.".into())
    })?;
    if envelope.anvil_problem != ENVELOPE_VERSION {
        return Err(AppError::Validation(format!(
            "Unsupported problem-file version {} — this build of Anvil reads version {}.",
            envelope.anvil_problem, ENVELOPE_VERSION
        )));
    }
    let mut problem = envelope.problem;
    problem
        .validate_structure()
        .map_err(|msg| AppError::Validation(format!("Invalid problem file: {msg}")))?;

    if id_exists(&problem.id) {
        let base = problem.id.clone();
        let mut n = 1;
        let mut candidate = format!("{base}-imported");
        while id_exists(&candidate) {
            n += 1;
            candidate = format!("{base}-imported-{n}");
        }
        problem.id = candidate;
        problem.title.push_str(" (imported)");
    }
    problem.source = ProblemSource::Imported;
    problem.number = next_number;
    Ok(problem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::problem::{Difficulty, FunctionSignature, Pattern, TestCase};

    fn sample() -> Problem {
        Problem {
            id: "fixture-roundtrip".into(),
            number: 7,
            title: "Round Trip Fixture".into(),
            pattern: Pattern("Greedy".into()),
            difficulty: Difficulty::Easy,
            source: ProblemSource::User,
            description_md: "Test fixture: return the sum of two integers.".into(),
            body_html: None,
            constraints: vec![],
            examples: vec![],
            function_signature: FunctionSignature {
                python: "def solve(a, b):\n    pass".into(),
                javascript: "function solve(a, b) {}".into(),
                extra: Default::default(),
            },
            checker: crate::domain::problem::Checker::Exact,
            judge: None,
            entry_point: None,
            test_cases: vec![
                TestCase {
                    input: vec![1.into(), 2.into()],
                    expected: 3.into(),
                    hidden: false,
                },
                TestCase {
                    input: vec![2.into(), 2.into()],
                    expected: 4.into(),
                    hidden: true,
                },
            ],
            hints: vec![],
            reference_solution: None,
            explanation_md: None,
            follow_up: None,
            license: "user-original".into(),
            author: "someone-else".into(),
        }
    }

    #[test]
    fn export_import_round_trips_modulo_source_and_number() {
        let original = sample();
        let json = export_envelope(&original, "2026-06-12T10:00:00").unwrap();
        let imported = parse_import(&json, |_| false, 42).unwrap();
        assert_eq!(imported.source, ProblemSource::Imported);
        assert_eq!(imported.number, 42);
        assert_eq!(imported.author, "someone-else"); // warranty carries
        let mut normalized = imported.clone();
        normalized.source = original.source;
        normalized.number = original.number;
        assert_eq!(normalized, original);
    }

    #[test]
    fn id_collision_gets_a_suffix_on_id_and_title() {
        let json = export_envelope(&sample(), "2026-06-12T10:00:00").unwrap();
        let imported = parse_import(&json, |id| id == "fixture-roundtrip", 42).unwrap();
        assert_eq!(imported.id, "fixture-roundtrip-imported");
        assert_eq!(imported.title, "Round Trip Fixture (imported)");

        // and the suffix itself can collide
        let imported2 = parse_import(
            &json,
            |id| id == "fixture-roundtrip" || id == "fixture-roundtrip-imported",
            43,
        )
        .unwrap();
        assert_eq!(imported2.id, "fixture-roundtrip-imported-2");
    }

    #[test]
    fn malformed_files_are_rejected_with_a_friendly_message() {
        let err = parse_import("{ not json", |_| false, 1).unwrap_err();
        assert!(err.to_string().contains("not a valid Anvil problem export"));
        // a bare problem without the envelope is also rejected
        let bare = serde_json::to_string(&sample()).unwrap();
        assert!(parse_import(&bare, |_| false, 1).is_err());
    }

    #[test]
    fn future_versions_are_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_str(&export_envelope(&sample(), "now").unwrap()).unwrap();
        value["anvil_problem"] = serde_json::json!(2);
        let err = parse_import(&value.to_string(), |_| false, 1).unwrap_err();
        assert!(err
            .to_string()
            .contains("Unsupported problem-file version 2"));
    }

    #[test]
    fn structurally_invalid_problems_are_rejected() {
        let mut bad = sample();
        bad.test_cases.retain(|tc| !tc.hidden);
        let json = export_envelope(&bad, "now").unwrap();
        let err = parse_import(&json, |_| false, 1).unwrap_err();
        assert!(err.to_string().contains("hidden"));
    }

    fn sample_with(id: &str) -> Problem {
        let mut p = sample();
        p.id = id.into();
        p
    }

    #[test]
    fn parse_any_handles_single_envelopes() {
        let json = export_envelope(&sample(), "now").unwrap();
        let imported = parse_any_import(&json, |_| false, 5).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].number, 5);
        assert_eq!(imported[0].source, ProblemSource::Imported);
    }

    #[test]
    fn pack_imports_all_problems_with_sequential_numbers() {
        let pack = export_pack(
            "Starter set",
            &[sample_with("p-a"), sample_with("p-b"), sample_with("p-c")],
            "now",
        )
        .unwrap();
        let imported = parse_any_import(&pack, |_| false, 10).unwrap();
        assert_eq!(imported.len(), 3);
        assert_eq!(
            imported.iter().map(|p| p.number).collect::<Vec<_>>(),
            vec![10, 11, 12]
        );
        assert!(imported.iter().all(|p| p.source == ProblemSource::Imported));
    }

    #[test]
    fn pack_members_dont_collide_with_each_other() {
        // two problems share an id within the pack
        let pack = export_pack("Dupes", &[sample_with("dup"), sample_with("dup")], "now").unwrap();
        let imported = parse_any_import(&pack, |_| false, 1).unwrap();
        assert_eq!(imported.len(), 2);
        let ids: Vec<&str> = imported.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["dup", "dup-imported"]);
    }

    #[test]
    fn pack_members_suffix_against_the_existing_library() {
        let pack =
            export_pack("Set", &[sample_with("known"), sample_with("fresh")], "now").unwrap();
        let imported = parse_any_import(&pack, |id| id == "known", 1).unwrap();
        assert_eq!(imported[0].id, "known-imported");
        assert_eq!(imported[1].id, "fresh");
    }

    #[test]
    fn empty_and_bad_packs_are_rejected() {
        let empty = export_pack("Empty", &[], "now").unwrap();
        assert!(parse_any_import(&empty, |_| false, 1).is_err());

        let mut value: serde_json::Value =
            serde_json::from_str(&export_pack("V", &[sample()], "now").unwrap()).unwrap();
        value["anvil_pack"] = serde_json::json!(99);
        let err = parse_any_import(&value.to_string(), |_| false, 1).unwrap_err();
        assert!(err.to_string().contains("Unsupported pack version 99"));
    }
}
