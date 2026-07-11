//! User-problem authoring types + pure validation (task 0010). Mirrors
//! `UserProblemDraft`/`ValidationResult` in `src/lib/types.ts`; the rule set
//! and its exact messages mirror the seam mock / `draft-validation.ts` —
//! the create page's sidebar displays them verbatim.

use serde::{Deserialize, Serialize};

use crate::domain::problem::{
    Difficulty, Example, FunctionSignature, Pattern, Problem, ProblemSource, ReferenceSolution,
    TestCase,
};
use crate::domain::run::CaseResult;
use crate::error::{AppError, AppResult};

/// Test case as typed in the form — raw JSON text, unparsed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftTestCase {
    pub input: String,
    pub expected: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProblemDraft {
    /// Present when editing an existing problem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    /// `Pattern | ""` on the TS side — plain string here, validated below.
    pub pattern: String,
    pub difficulty: Difficulty,
    pub description_md: String,
    pub constraints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<String>,
    pub examples: Vec<Example>,
    pub function_signature: FunctionSignature,
    pub test_cases: Vec<DraftTestCase>,
    pub hints: Vec<String>,
    pub reference_solution: ReferenceSolution,
    /// Legal requirement (PROJECT_SPEC §8.5) — must be true to save.
    #[serde(rename = "originalityWarranty")]
    pub originality_warranty: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
    #[serde(rename = "caseResults", skip_serializing_if = "Option::is_none")]
    pub case_results: Option<Vec<CaseResult>>,
}

/// Drafts list row for the create page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftSummary {
    pub id: String,
    pub title: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

fn issue(field: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        field: field.into(),
        message: message.into(),
    }
}

/// Pure rule set — message-identical with the mock seam (and the create
/// page's live `draft-validation.ts`).
pub fn validate(draft: &UserProblemDraft) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if draft.title.trim().is_empty() {
        issues.push(issue("Title", "is required"));
    }
    if !Pattern(draft.pattern.clone()).is_known() {
        issues.push(issue("Pattern", "pick one of the 15 patterns"));
    }
    if draft.description_md.trim().is_empty() {
        issues.push(issue("Description", "is required"));
    }
    if draft.examples.is_empty() {
        issues.push(issue("Examples", "at least one is required"));
    }
    for (i, ex) in draft.examples.iter().enumerate() {
        if ex.input.trim().is_empty() || ex.output.trim().is_empty() {
            issues.push(issue(
                &format!("Example {}", i + 1),
                "needs both input and output",
            ));
        }
    }
    if draft.function_signature.python.trim().is_empty() {
        issues.push(issue("Python signature", "is empty"));
    }
    if draft.function_signature.javascript.trim().is_empty() {
        issues.push(issue("JavaScript signature", "is empty"));
    }

    let mut visible = 0;
    let mut hidden = 0;
    for (i, tc) in draft.test_cases.iter().enumerate() {
        let field = format!("Test case {}", i + 1);
        match serde_json::from_str::<serde_json::Value>(&tc.input) {
            Ok(value) if !value.is_array() => {
                issues.push(issue(&field, "Input must be a JSON array of arguments"));
            }
            Ok(_) => {}
            Err(_) => issues.push(issue(&field, "Input isn't valid JSON")),
        }
        if serde_json::from_str::<serde_json::Value>(&tc.expected).is_err() {
            issues.push(issue(&field, "Expected isn't valid JSON"));
        }
        if tc.hidden {
            hidden += 1;
        } else {
            visible += 1;
        }
    }
    if visible < 1 {
        issues.push(issue("Test cases", "need at least 1 visible test"));
    }
    if hidden < 1 {
        issues.push(issue("Test cases", "need at least 1 hidden test"));
    }
    if !draft.originality_warranty {
        issues.push(issue("Originality warranty", "must be accepted to save"));
    }
    issues
}

/// Builds the full `Problem` record (spec §8.2: a user problem is just
/// another record). Call after `validate` is clean; parse failures here are
/// defensive.
pub fn build_problem(draft: &UserProblemDraft, id: &str, number: u32) -> AppResult<Problem> {
    let mut test_cases = Vec::with_capacity(draft.test_cases.len());
    for (i, tc) in draft.test_cases.iter().enumerate() {
        let input: Vec<serde_json::Value> = serde_json::from_str(&tc.input).map_err(|_| {
            AppError::Validation(format!("Test case {}: Input isn't valid JSON", i + 1))
        })?;
        let expected: serde_json::Value = serde_json::from_str(&tc.expected).map_err(|_| {
            AppError::Validation(format!("Test case {}: Expected isn't valid JSON", i + 1))
        })?;
        test_cases.push(TestCase {
            input,
            expected,
            hidden: tc.hidden,
        });
    }
    Ok(Problem {
        id: id.to_string(),
        number,
        title: draft.title.trim().to_string(),
        pattern: Pattern(draft.pattern.clone()),
        difficulty: draft.difficulty,
        source: ProblemSource::User,
        description_md: draft.description_md.clone(),
        body_html: None,
        constraints: draft
            .constraints
            .iter()
            .filter(|c| !c.trim().is_empty())
            .cloned()
            .collect(),
        examples: draft.examples.clone(),
        function_signature: draft.function_signature.clone(),
        test_cases,
        checker: crate::domain::problem::Checker::Exact,
        judge: None,
        entry_point: None,
        hints: draft
            .hints
            .iter()
            .filter(|h| !h.trim().is_empty())
            .cloned()
            .collect(),
        reference_solution: Some(draft.reference_solution.clone()),
        explanation_md: None,
        follow_up: draft
            .follow_up
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from),
        license: "user-original".into(),
        author: "user".into(),
    })
}

/// Slugified title + short time-derived suffix, like the mock's id rule.
pub fn new_problem_id(title: &str, suffix: u64) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug = if slug.is_empty() {
        "problem".to_string()
    } else {
        slug
    };
    format!("{slug}-{}", suffix % 10000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_draft() -> UserProblemDraft {
        serde_json::from_value(json!({
            "title": "Sum Of Two",
            "pattern": "Greedy",
            "difficulty": "Easy",
            "description_md": "Return the sum of two integers.",
            "constraints": ["-100 <= a, b <= 100", ""],
            "examples": [{ "input": "a = 1, b = 2", "output": "3" }],
            "function_signature": {
                "python": "def solve(a, b):\n    pass",
                "javascript": "function solve(a, b) {}"
            },
            "test_cases": [
                { "input": "[1, 2]", "expected": "3", "hidden": false },
                { "input": "[5, 5]", "expected": "10", "hidden": true }
            ],
            "hints": ["Use +", ""],
            "reference_solution": { "python": "def solve(a, b):\n    return a + b" },
            "originalityWarranty": true
        }))
        .unwrap()
    }

    #[test]
    fn a_valid_draft_produces_no_issues() {
        assert_eq!(validate(&valid_draft()), vec![]);
    }

    #[test]
    fn each_rule_uses_the_mock_exact_message() {
        let mut d = valid_draft();
        d.title = " ".into();
        d.pattern = "Not A Pattern".into();
        d.description_md = String::new();
        d.examples = vec![];
        d.function_signature.python = String::new();
        d.function_signature.javascript = "  ".into();
        d.test_cases = vec![DraftTestCase {
            input: "{not json".into(),
            expected: "nope".into(),
            hidden: false,
        }];
        d.originality_warranty = false;

        let issues = validate(&d);
        let pairs: Vec<(String, String)> = issues
            .iter()
            .map(|i| (i.field.clone(), i.message.clone()))
            .collect();
        let expect = [
            ("Title", "is required"),
            ("Pattern", "pick one of the 15 patterns"),
            ("Description", "is required"),
            ("Examples", "at least one is required"),
            ("Python signature", "is empty"),
            ("JavaScript signature", "is empty"),
            ("Test case 1", "Input isn't valid JSON"),
            ("Test case 1", "Expected isn't valid JSON"),
            ("Test cases", "need at least 1 hidden test"),
            ("Originality warranty", "must be accepted to save"),
        ];
        for (field, message) in expect {
            assert!(
                pairs.contains(&(field.to_string(), message.to_string())),
                "missing issue {field}: {message} in {pairs:?}"
            );
        }
    }

    #[test]
    fn non_array_input_is_rejected_with_the_live_validation_message() {
        let mut d = valid_draft();
        d.test_cases[0].input = "{\"a\": 1}".into();
        let issues = validate(&d);
        assert_eq!(issues[0].field, "Test case 1");
        assert_eq!(issues[0].message, "Input must be a JSON array of arguments");
    }

    #[test]
    fn build_problem_parses_cases_and_trims_optionals() {
        let p = build_problem(&valid_draft(), "sum-of-two-123", 13).unwrap();
        assert_eq!(p.number, 13);
        assert_eq!(p.source, ProblemSource::User);
        assert_eq!(p.license, "user-original");
        assert_eq!(p.test_cases[0].input, vec![json!(1), json!(2)]);
        assert_eq!(p.test_cases[1].expected, json!(10));
        assert_eq!(p.constraints.len(), 1); // empty constraint dropped
        assert_eq!(p.hints.len(), 1); // empty hint dropped
        assert!(p.follow_up.is_none());
    }

    #[test]
    fn ids_are_slugged_with_a_suffix() {
        assert_eq!(new_problem_id("Sum Of Two!", 91234), "sum-of-two-1234");
        assert_eq!(new_problem_id("---", 7), "problem-7");
    }

    #[test]
    fn validation_result_serializes_like_types_ts() {
        let r = ValidationResult {
            ok: true,
            issues: vec![],
            case_results: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v, json!({ "ok": true, "issues": [] }));
        let with_issue = ValidationResult {
            ok: false,
            issues: vec![ValidationIssue {
                field: "Title".into(),
                message: "is required".into(),
            }],
            case_results: None,
        };
        assert_eq!(
            serde_json::to_value(&with_issue).unwrap(),
            json!({ "ok": false, "issues": [{ "field": "Title", "message": "is required" }] })
        );
    }

    #[test]
    fn draft_round_trips_with_camel_case_warranty() {
        let d = valid_draft();
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["originalityWarranty"], json!(true));
        let back: UserProblemDraft = serde_json::from_value(v).unwrap();
        assert_eq!(back, d);
    }
}
