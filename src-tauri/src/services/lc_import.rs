//! LeetCode import logic (task 0005): parse the
//! scrape, match each question to its bundled pack by slug, verify the match
//! by entry-point fingerprint, and merge the user's statement with our pack
//! into the unified `Problem` schema. Tauri-free and runtime-free — stress
//! materialization (the only step needing the sandbox) is done by the caller
//! and passed in, so the whole match/merge layer unit-tests as plain Rust.
//!
//! Privacy/legal: the scrape is the user's own data; it is merged in full and
//! written only to the local DB (`commands/import.rs`). Pack content is ours.

use std::collections::HashSet;

use crate::domain::lc_import::{
    pattern_from_topics, ImportTier, PreviewItem, ScrapeFile, ScrapeQuestion, PATTERN_FALLBACK,
};
use crate::domain::pack::TestPack;
use crate::domain::preset::Preset;
use crate::domain::problem::{
    Complexity, EntryPoint, Example, FunctionSignature, Pattern, Problem, ProblemSource,
    ReferenceSolution, TestCase,
};
use crate::error::{AppError, AppResult};
use crate::services::example_parse::{self, parse_examples};

/// Parses + validates the scrape envelope. Rejects with a precise message the
/// UI shows verbatim (an empty/garbage file must not look like "0 imported").
pub fn parse_scrape(json: &str) -> AppResult<ScrapeFile> {
    let scrape: ScrapeFile = serde_json::from_str(json).map_err(|e| {
        AppError::Validation(format!(
            "This file isn't a valid scrape export (expected the JSON produced by the \
             scraper tool): {e}"
        ))
    })?;
    if scrape.questions.is_empty() {
        return Err(AppError::Validation(
            "This scrape file contains no questions.".into(),
        ));
    }
    Ok(scrape)
}

/// Builds the pre-import preview / selection list. `has_pack` is the full-tier
/// signal; `already_imported` flags re-import candidates.
pub fn build_preview_items(
    scrape: &ScrapeFile,
    has_pack: impl Fn(&str) -> bool,
    already_imported: &HashSet<String>,
) -> Vec<PreviewItem> {
    scrape
        .questions
        .iter()
        .map(|q| PreviewItem {
            slug: q.slug.clone(),
            title: q.title.clone(),
            difficulty: q.difficulty,
            has_pack: has_pack(&q.slug),
            already_imported: already_imported.contains(&q.slug),
            is_premium: q.is_premium,
        })
        .collect()
}

/// Allowlist-sanitized statement HTML. Uses ammonia's
/// safe default (scripts, event handlers, `javascript:` URLs all stripped)
/// and additionally drops `img` — the statement should never pull remote
/// assets in an offline app.
pub fn sanitize_html(raw: &str) -> String {
    ammonia::Builder::default()
        .rm_tags(["img"])
        .clean(raw)
        .to_string()
}

/// Verifies a pack actually belongs to this scraped question: the pack's
/// entry-point method name must appear in the imported Python stub and, when
/// the stub's arity is determinable, match it. A
/// mismatch means LeetCode changed the problem or the match is wrong → the
/// caller demotes to basic mode rather than judge against the wrong tests.
pub fn verify_fingerprint(pack_entry: &EntryPoint, python_stub: &str) -> bool {
    let method = pack_entry.python.rsplit('.').next().unwrap_or("");
    if method.is_empty() {
        return false;
    }
    if first_def_name(python_stub).as_deref() != Some(method) {
        // The stub's first def must be the entry method (LeetCode stubs lead
        // with the solution method).
        if !python_stub.contains(&format!("def {method}(")) {
            return false;
        }
    }
    match example_parse::python_stub_arity(python_stub) {
        Some(arity) => arity as u32 == pack_entry.arity,
        // Unparseable arity ⇒ trust the name match alone.
        None => true,
    }
}

/// Entry point derived from the question's own stub, so a basic-mode problem
/// (no pack) still runs the user's `class Solution`/named function unmodified.
pub fn derive_entry_point(python_stub: &str, javascript_stub: &str) -> Option<EntryPoint> {
    let method = first_def_name(python_stub)?;
    let python = match example_parse::stub_class_name(python_stub) {
        Some(class) => format!("{class}.{method}"),
        None => method.clone(),
    };
    let javascript = js_function_name(javascript_stub).unwrap_or_else(|| method.clone());
    let arity = example_parse::python_stub_arity(python_stub).unwrap_or(0) as u32;
    Some(EntryPoint {
        python,
        javascript,
        arity,
        // Basic-mode derivation has no pack, so no node I/O types; a full pack
        // carries its own `io_types` from the bundle.
        io_types: None,
    })
}

/// The Anvil pattern for a question: the curated preset grouping wins; then
/// the topic-slug heuristic; else the fallback bucket (with a note). Returns
/// `(pattern, fallback_note)`.
pub fn resolve_pattern(
    slug: &str,
    topic_slugs: &[String],
    presets: &[Preset],
) -> (Pattern, Option<String>) {
    for preset in presets {
        if let Some(pattern) = preset.pattern_of(slug) {
            return (pattern.clone(), None);
        }
    }
    if let Some(pattern) = pattern_from_topics(topic_slugs) {
        return (pattern, None);
    }
    (
        Pattern(PATTERN_FALLBACK.to_string()),
        Some(format!(
            "'{slug}': no pattern from topics {topic_slugs:?} — filed under {PATTERN_FALLBACK}"
        )),
    )
}

/// Preset ids that list this slug, for the Library preset filter.
pub fn preset_tags(slug: &str, presets: &[Preset]) -> Vec<String> {
    presets
        .iter()
        .filter(|p| p.contains(slug))
        .map(|p| p.id.clone())
        .collect()
}

/// The outcome of merging one scraped question with its (verified) pack.
#[derive(Debug, Clone, PartialEq)]
pub struct Merged {
    pub problem: Problem,
    pub tier: ImportTier,
    pub presets: Vec<String>,
    /// Non-fatal notes worth surfacing (pattern fallback, example drops).
    pub notes: Vec<String>,
}

/// Merges a scraped question and (optionally) its verified pack into a
/// `Problem`. `verified_pack` is `Some` only when the slug matched AND the
/// fingerprint passed; otherwise the question lands in basic / run-only mode.
/// `materialized_stress` is the pack's stress specs already executed into
/// literal hidden cases (empty when no pack / no runtime).
pub fn merge_question(
    q: &ScrapeQuestion,
    verified_pack: Option<&TestPack>,
    materialized_stress: Vec<TestCase>,
    presets: &[Preset],
    number: u32,
) -> Merged {
    let mut notes = Vec::new();

    let (pattern, fallback_note) = resolve_pattern(&q.slug, &q.topic_slugs, presets);
    if let Some(note) = fallback_note {
        notes.push(note);
    }
    let preset_ids = preset_tags(&q.slug, presets);

    // Visible tests = the statement's own examples.
    let input_lines: Vec<Vec<String>> = q
        .example_tests
        .iter()
        .map(|e| e.input_lines.clone())
        .collect();
    let parsed = parse_examples(&input_lines, &q.body_text, &q.code_stubs.python);
    if !parsed.report.dropped.is_empty() {
        notes.push(format!(
            "'{}': {} example(s) not runnable ({})",
            q.slug,
            parsed.report.dropped.len(),
            parsed.report.dropped.join("; ")
        ));
    }
    let mut visible = parsed.cases;

    // Tier + the pack-derived fields.
    let (tier, judge, entry_point, hints, reference_solution, explanation_md) = match verified_pack
    {
        Some(pack) => {
            // Packs whose wire format differs from the statement's example
            // encoding (closing-the-48: cyclic lists, multilevel lists, shim
            // args, …) can't use the parsed examples — the inputs wouldn't
            // deserialize and the expected values wouldn't compare. The build
            // recorded this as `examples_ok = false`; drop them here.
            if !pack.examples_ok && !visible.is_empty() {
                notes.push(format!(
                    "'{}': statement examples replaced by pack cases (wire format differs)",
                    q.slug
                ));
                visible.clear();
            }
            // If no statement example survived parsing (design-shaped
            // statements, multi-variable inputs) promote the pack's leading
            // tests instead, so Run always has real, wire-correct cases.
            let promote = if visible.is_empty() {
                pack.tests.len().min(2)
            } else {
                0
            };
            for (i, t) in pack.tests.iter().enumerate() {
                visible.push(TestCase {
                    input: t.input.clone(),
                    expected: t.expected.clone(),
                    hidden: i >= promote,
                });
            }
            visible.extend(materialized_stress);
            let reference = ReferenceSolution {
                python: Some(pack.solutions.python.clone()),
                javascript: pack.solutions.javascript.clone(),
                complexity: pack.solutions.complexity.clone().map(|c| Complexity {
                    time: c.time,
                    space: c.space,
                }),
            };
            (
                ImportTier::Full,
                Some(pack.judge.clone()),
                Some(pack.entry_point.clone()),
                pack.hints.clone(),
                Some(reference),
                Some(pack.pattern.clone()),
            )
        }
        None => {
            // Basic if we recovered runnable examples, else run-only.
            let tier = if visible.iter().any(|c| !c.hidden) {
                ImportTier::Basic
            } else {
                ImportTier::RunOnly
            };
            (
                tier,
                None,
                derive_entry_point(&q.code_stubs.python, &q.code_stubs.javascript),
                q.hints.clone(),
                None,
                None,
            )
        }
    };

    let body_html = if q.body_html.trim().is_empty() {
        None
    } else {
        Some(sanitize_html(&q.body_html))
    };

    let problem = Problem {
        id: q.slug.clone(),
        number,
        title: if q.title.trim().is_empty() {
            q.slug.clone()
        } else {
            q.title.clone()
        },
        pattern,
        difficulty: q.difficulty,
        source: ProblemSource::Imported,
        // body_text is the search/fallback source; the panel renders body_html.
        description_md: q.body_text.clone(),
        body_html,
        constraints: extract_constraints(&q.body_text),
        // The HTML statement carries the examples; no separate display copies.
        examples: Vec::<Example>::new(),
        function_signature: FunctionSignature {
            python: q.code_stubs.python.clone(),
            javascript: q.code_stubs.javascript.clone(),
            // Preserve any additional languages the catalog ships (cpp, java, …)
            // so a richer re-import doesn't silently drop them.
            extra: q.code_stubs.extra.clone(),
        },
        test_cases: visible,
        checker: crate::domain::problem::Checker::Exact,
        judge,
        entry_point,
        hints,
        reference_solution,
        explanation_md,
        follow_up: None,
        license: "imported-leetcode".into(),
        author: "leetcode".into(),
    };

    Merged {
        problem,
        tier,
        presets: preset_ids,
        notes,
    }
}

/// Pulls the `Constraints:` block out of the plain-text statement into bullet
/// strings (backticks stripped). Best-effort — empty when none are present.
fn extract_constraints(body_text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_block = false;
    for line in body_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Constraints") {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        // Stop at the next section heading.
        if trimmed.starts_with("Follow-up") || trimmed.starts_with("Example") {
            break;
        }
        let cleaned = trimmed.trim_matches('`').trim();
        if !cleaned.is_empty() {
            out.push(cleaned.to_string());
        }
    }
    out
}

/// First `def NAME(` identifier in a Python stub.
fn first_def_name(python_stub: &str) -> Option<String> {
    // Decomment first so the commented-out node-class prelude (`# def __init__`)
    // doesn't shadow the real solution method.
    let stub = example_parse::decomment_python(python_stub);
    let idx = stub.find("def ")?;
    let rest = &stub[idx + 4..];
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// The function identifier a JS stub binds, covering the forms LeetCode emits:
/// `var f = function`, `const f = (…) =>`, `function f(…)`.
fn js_function_name(js_stub: &str) -> Option<String> {
    for line in js_stub.lines() {
        let t = line.trim();
        for kw in ["var ", "let ", "const "] {
            if let Some(rest) = t.strip_prefix(kw) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                    .collect();
                if !name.is_empty() && rest[name.len()..].trim_start().starts_with('=') {
                    return Some(name);
                }
            }
        }
        if let Some(rest) = t.strip_prefix("function ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pack::{PackSolutions, PackTest, PackTestKind};
    use crate::domain::problem::{Difficulty, Judge};
    use serde_json::json;

    const PY_STUB: &str =
        "class Solution:\n    def twoSum(self, nums: List[int], target: int) -> List[int]:\n        ";
    const JS_STUB: &str = "var twoSum = function(nums, target) {\n};";

    fn question() -> ScrapeQuestion {
        serde_json::from_value(json!({
            "qid": "1", "slug": "two-sum", "title": "Two Sum", "difficulty": "Easy",
            "is_premium": "false",
            "body_html": "<p>Given <code>nums</code><script>alert(1)</script></p>",
            "body_text": "Given nums and target.\n\nExample 1:\n\nInput: nums = [2,7,11,15], target = 9\nOutput: [0,1]\n\nConstraints:\n\n`2 <= nums.length <= 10000`\n`-10^9 <= target <= 10^9`\n",
            "topic_slugs": ["array", "hash-table"],
            "code_stubs": { "python": PY_STUB, "javascript": JS_STUB },
            "example_tests": [{ "input_lines": ["[2,7,11,15]", "9"] }],
            "scraped_at": "2026-06-12T00:00:00Z"
        }))
        .unwrap()
    }

    fn pack() -> TestPack {
        TestPack {
            slug: "two-sum".into(),
            qid: "1".into(),
            schema_version: 1,
            entry_point: EntryPoint {
                python: "Solution.twoSum".into(),
                javascript: "twoSum".into(),
                arity: 2,
                io_types: None,
            },
            judge: Judge::Exact,
            pattern: "Hash map of complements.".into(),
            hints: vec!["nudge".into(), "approach".into(), "answer".into()],
            constraints: vec![],
            tests: vec![PackTest {
                kind: PackTestKind::Edge,
                description: "dupes".into(),
                input: vec![json!([3, 3]), json!(6)],
                expected: json!([0, 1]),
            }],
            stress: vec![],
            solutions: PackSolutions {
                python: "class Solution:\n    def twoSum(self, nums, target): ...".into(),
                javascript: Some("var twoSum = function(nums, target) {};".into()),
                brute_force_python: None,
                complexity: None,
            },
            examples_ok: true,
            verified: true,
            generated_at: "2026-06-12T00:00:00Z".into(),
        }
    }

    #[test]
    fn html_sanitizer_strips_scripts_and_images() {
        let clean = sanitize_html("<p>ok<script>alert(1)</script><img src=x onerror=y></p>");
        assert!(clean.contains("<p>"));
        assert!(clean.contains("ok"));
        assert!(!clean.contains("script"));
        assert!(!clean.contains("img"));
        assert!(!clean.contains("onerror"));
    }

    #[test]
    fn fingerprint_matches_name_and_arity() {
        let ep = pack().entry_point;
        assert!(verify_fingerprint(&ep, PY_STUB));
        // wrong arity is rejected
        let mut bad = ep.clone();
        bad.arity = 3;
        assert!(!verify_fingerprint(&bad, PY_STUB));
        // wrong method name is rejected
        let mut renamed = ep.clone();
        renamed.python = "Solution.threeSum".into();
        assert!(!verify_fingerprint(&renamed, PY_STUB));
    }

    #[test]
    fn derives_entry_point_from_stub_for_basic_mode() {
        let ep = derive_entry_point(PY_STUB, JS_STUB).unwrap();
        assert_eq!(ep.python, "Solution.twoSum");
        assert_eq!(ep.javascript, "twoSum");
        assert_eq!(ep.arity, 2);
        // bare function + arrow JS
        let ep =
            derive_entry_point("def reverse(s):\n    pass", "const reverse = (s) => s;").unwrap();
        assert_eq!(ep.python, "reverse");
        assert_eq!(ep.javascript, "reverse");
    }

    #[test]
    fn incompatible_statement_examples_are_replaced_by_pack_cases() {
        // closing-the-48: a pack whose wire format differs from the statement
        // encoding drops the parsed examples and promotes its own leading
        // tests to visible, so Run always has wire-correct cases.
        let mut p = pack();
        p.examples_ok = false;
        let merged = merge_question(&question(), Some(&p), vec![], &[], 5000);
        assert_eq!(merged.tier, ImportTier::Full);
        let visible: Vec<_> = merged
            .problem
            .test_cases
            .iter()
            .filter(|c| !c.hidden)
            .collect();
        // The statement example is gone; the pack's (single) test is visible.
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].input, pack().tests[0].input);
        assert!(merged
            .notes
            .iter()
            .any(|n| n.contains("statement examples replaced")));
    }

    #[test]
    fn full_tier_merges_pack_as_hidden_judge() {
        let presets = vec![];
        let merged = merge_question(&question(), Some(&pack()), vec![], &presets, 5000);
        assert_eq!(merged.tier, ImportTier::Full);
        assert_eq!(merged.problem.id, "two-sum");
        assert_eq!(merged.problem.source, ProblemSource::Imported);
        // one visible example + one hidden pack test
        let visible = merged
            .problem
            .test_cases
            .iter()
            .filter(|c| !c.hidden)
            .count();
        let hidden = merged
            .problem
            .test_cases
            .iter()
            .filter(|c| c.hidden)
            .count();
        assert_eq!(visible, 1);
        assert_eq!(hidden, 1);
        assert_eq!(merged.problem.judge, Some(Judge::Exact));
        assert_eq!(
            merged.problem.entry_point.as_ref().unwrap().python,
            "Solution.twoSum"
        );
        assert!(merged.problem.reference_solution.is_some());
        assert_eq!(
            merged.problem.explanation_md.as_deref(),
            Some("Hash map of complements.")
        );
        // body_html sanitized + present; constraints extracted
        assert!(merged.problem.body_html.as_ref().unwrap().contains("<p>"));
        assert!(!merged
            .problem
            .body_html
            .as_ref()
            .unwrap()
            .contains("script"));
        assert_eq!(merged.problem.constraints.len(), 2);
        assert_eq!(merged.problem.difficulty, Difficulty::Easy);
    }

    #[test]
    fn basic_tier_when_no_pack_uses_examples_and_derived_entry_point() {
        let merged = merge_question(&question(), None, vec![], &[], 5000);
        assert_eq!(merged.tier, ImportTier::Basic);
        assert!(merged.problem.judge.is_none());
        assert!(merged.problem.entry_point.is_some());
        assert_eq!(
            merged
                .problem
                .test_cases
                .iter()
                .filter(|c| c.hidden)
                .count(),
            0
        );
        assert!(merged.problem.reference_solution.is_none());
    }

    #[test]
    fn run_only_when_examples_unparseable_and_no_pack() {
        let mut q = question();
        q.example_tests.clear();
        q.body_text = "No examples here.".into();
        let merged = merge_question(&q, None, vec![], &[], 5000);
        assert_eq!(merged.tier, ImportTier::RunOnly);
        assert!(merged.problem.test_cases.is_empty());
    }

    #[test]
    fn extra_language_stubs_survive_the_merge() {
        // A catalog that ships languages beyond Python/JS must keep them:
        // they land in `function_signature.extra`, verbatim.
        let q: ScrapeQuestion = serde_json::from_value(json!({
            "slug": "two-sum", "title": "Two Sum", "difficulty": "Easy",
            "body_text": "Given nums.\n\nExample 1:\n\nInput: nums = [1]\nOutput: [0]\n",
            "code_stubs": {
                "python": PY_STUB,
                "javascript": JS_STUB,
                "cpp": "class Solution {\npublic:\n    vector<int> twoSum(...) {}\n};",
                "java": "class Solution {\n    int[] twoSum(...) {}\n}"
            },
            "example_tests": [{ "input_lines": ["[1]"] }]
        }))
        .unwrap();
        assert_eq!(q.code_stubs.extra.len(), 2);
        let merged = merge_question(&q, None, vec![], &[], 1);
        let sig = &merged.problem.function_signature;
        assert!(sig.python.contains("twoSum"));
        assert_eq!(
            sig.extra.get("cpp").map(String::as_str),
            Some("class Solution {\npublic:\n    vector<int> twoSum(...) {}\n};")
        );
        assert!(sig.extra.contains_key("java"));
        // and the two runnable languages are NOT duplicated into `extra`
        assert!(!sig.extra.contains_key("python"));
        assert!(!sig.extra.contains_key("javascript"));
    }

    #[test]
    fn parse_scrape_rejects_empty_and_garbage() {
        assert!(parse_scrape("not json").is_err());
        assert!(parse_scrape(r#"{"questions": []}"#).is_err());
        let ok =
            parse_scrape(r#"{"schema_version":"3","questions":[{"slug":"two-sum"}]}"#).unwrap();
        assert_eq!(ok.questions.len(), 1);
    }
}
