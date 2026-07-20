//! Statement example parser (task 0004):
//! turns a scraped question's own `Example` blocks into runnable visible
//! `TestCase`s — the basic-mode tests for unmatched questions and the
//! ground-truth anchor for the generation pipeline (whose Python mirror in
//! `tools/generate_test_packs.py` must follow the same rules).
//!
//! Tauri-free: plain inputs (`input_lines`, `body_text`, the Python stub),
//! plain outputs. The rule throughout is **never guess** — anything
//! ambiguous is dropped per-example with a reason, and a question whose
//! examples all drop becomes run-only.

use serde_json::Value;

use crate::domain::problem::TestCase;

/// Outcome of parsing one question's statement examples.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedExamples {
    /// All visible (`hidden: false`), paired input → expected.
    pub cases: Vec<TestCase>,
    pub report: ParseReport,
}

/// Per-question parse accounting, surfaced in the import summary.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParseReport {
    /// Example blocks found in the statement.
    pub total: usize,
    /// Examples that became test cases.
    pub parsed: usize,
    /// Per-example (or whole-question) drop reasons.
    pub dropped: Vec<String>,
}

/// Parses the statement's examples into visible test cases.
///
/// - `input_lines`: per example, the structured input lines from the scrape
///   (`example_tests[].input_lines`, one JSON value per parameter). Empty ⇒
///   fall back to parsing `Input: name = …` from the statement.
/// - `body_text`: plain-text statement; the only source of `Output:` values.
/// - `python_stub`: the question's own Python stub — parameter order/arity.
pub fn parse_examples(
    input_lines: &[Vec<String>],
    body_text: &str,
    python_stub: &str,
) -> ParsedExamples {
    let mut report = ParseReport::default();

    // Design-style problems (class stub that isn't `Solution`) use the ops
    // wire format; their examples are handled by the pack path, never here.
    if let Some(class_name) = stub_class_name(python_stub) {
        if class_name != "Solution" {
            report.dropped.push(format!(
                "design-style problem (class '{class_name}') — statement examples are not \
                 plain input/output pairs"
            ));
            return ParsedExamples {
                cases: vec![],
                report,
            };
        }
    }

    let arity = python_stub_arity(python_stub);

    let blocks = example_blocks(body_text);
    report.total = blocks.len();

    // Inputs: structured lines when present, else the statement's `Input:`.
    let inputs: Vec<Option<Vec<Value>>> = if !input_lines.is_empty() {
        input_lines
            .iter()
            .map(|lines| {
                lines
                    .iter()
                    .map(|line| serde_json::from_str::<Value>(line.trim()).ok())
                    .collect::<Option<Vec<Value>>>()
            })
            .collect()
    } else {
        blocks
            .iter()
            .map(|b| b.input.as_deref().and_then(parse_input_expression))
            .collect()
    };

    let outputs: Vec<Option<Value>> = blocks
        .iter()
        .map(|b| {
            b.output
                .as_deref()
                .and_then(|s| serde_json::from_str::<Value>(s.trim()).ok())
        })
        .collect();

    // Pair by index. A count mismatch means we cannot pair confidently —
    // drop everything rather than guess.
    if inputs.len() != outputs.len() {
        report.dropped.push(format!(
            "example count mismatch: {} inputs vs {} statement examples",
            inputs.len(),
            outputs.len()
        ));
        return ParsedExamples {
            cases: vec![],
            report,
        };
    }

    let mut cases = Vec::new();
    for (i, (input, output)) in inputs.into_iter().zip(outputs).enumerate() {
        let n = i + 1;
        let Some(args) = input else {
            report
                .dropped
                .push(format!("example {n}: input failed to parse as JSON"));
            continue;
        };
        let Some(expected) = output else {
            report
                .dropped
                .push(format!("example {n}: no parseable `Output:` value"));
            continue;
        };
        if let Some(arity) = arity {
            if args.len() != arity {
                report.dropped.push(format!(
                    "example {n}: {} argument(s) but the stub takes {arity}",
                    args.len()
                ));
                continue;
            }
        }
        cases.push(TestCase {
            input: args,
            expected,
            hidden: false,
        });
    }

    report.parsed = cases.len();
    ParsedExamples { cases, report }
}

/// Strip Python comments and triple-quoted blocks. LeetCode ships node-problem
/// stubs with a commented-out `# class ListNode:` / `# def __init__(...)`
/// prelude (or a `"""..."""` block for graph `Node`); without this, the first
/// `def`/`class`/arity scan latches onto `__init__` instead of the real method.
pub fn decomment_python(python_stub: &str) -> String {
    let mut s = python_stub.to_string();
    for quote in ["\"\"\"", "'''"] {
        while let Some(start) = s.find(quote) {
            match s[start + 3..].find(quote) {
                Some(end_rel) => s.replace_range(start..start + 3 + end_rel + 3, ""),
                None => break,
            }
        }
    }
    s.lines()
        .filter(|ln| !ln.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The class a Python stub defines, if any (`class Solution:` → "Solution").
pub fn stub_class_name(python_stub: &str) -> Option<String> {
    for line in decomment_python(python_stub).lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("class ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

/// Parameter count of the stub's first method/function, excluding `self`.
/// `None` when no `def` is found (arity can't be validated).
pub fn python_stub_arity(python_stub: &str) -> Option<usize> {
    python_stub_params(python_stub).map(|p| p.len())
}

/// Parameter names of the stub's first method/function, excluding `self`.
pub fn python_stub_params(python_stub: &str) -> Option<Vec<String>> {
    let decommented = decomment_python(python_stub);
    let python_stub = decommented.as_str();
    let def = python_stub.find("def ")?;
    let after = &python_stub[def..];
    let open = after.find('(')?;
    let close = matching_paren(&after[open..])? + open;
    let inner = &after[open + 1..close];
    Some(
        split_top_level(inner)
            .into_iter()
            .filter_map(|part| {
                let name = part.split(':').next().unwrap_or("").trim();
                (!name.is_empty() && name != "self").then(|| name.to_string())
            })
            .collect(),
    )
}

/// Index of the `)` matching the `(` at byte 0 of `s` (handles nesting in
/// type annotations like `dict[str, int]` — brackets all count).
fn matching_paren(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// One `Example N:` block lifted from the statement text.
#[derive(Debug, Default)]
struct ExampleBlock {
    /// Text after `Input:` on its line (single-line form only).
    input: Option<String>,
    /// Text after `Output:` on its line.
    output: Option<String>,
}

/// Splits the statement into example blocks. A block starts at a line
/// beginning with `Example` and ends at the next `Example` line,
/// `Constraints` line, or end of text.
fn example_blocks(body_text: &str) -> Vec<ExampleBlock> {
    let mut blocks: Vec<ExampleBlock> = Vec::new();
    let mut current: Option<ExampleBlock> = None;
    for line in body_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Example") {
            if let Some(done) = current.take() {
                blocks.push(done);
            }
            current = Some(ExampleBlock::default());
            continue;
        }
        if trimmed.starts_with("Constraints") {
            break;
        }
        if let Some(block) = current.as_mut() {
            if let Some(rest) = trimmed.strip_prefix("Input:") {
                block.input.get_or_insert_with(|| rest.trim().to_string());
            } else if let Some(rest) = trimmed.strip_prefix("Output:") {
                block.output.get_or_insert_with(|| rest.trim().to_string());
            }
        }
    }
    if let Some(done) = current.take() {
        blocks.push(done);
    }
    blocks
}

/// Parses the statement form `nums = [2,7,11,15], target = 9` into
/// positional JSON args: split on top-level commas, strip `name =`
/// prefixes, parse each as JSON. Any failure ⇒ `None` (never guess).
fn parse_input_expression(input: &str) -> Option<Vec<Value>> {
    let parts = split_top_level(input);
    if parts.is_empty() {
        return None;
    }
    parts
        .into_iter()
        .map(|part| {
            let value_text = match part.split_once('=') {
                // `name = value` — but only when the left side looks like an
                // identifier; `=` inside a value never splits because
                // split_top_level already isolated this argument.
                Some((name, value))
                    if !name.trim().is_empty()
                        && name.trim().chars().all(|c| c.is_alphanumeric() || c == '_') =>
                {
                    value.trim()
                }
                _ => part.trim(),
            };
            serde_json::from_str::<Value>(value_text).ok()
        })
        .collect()
}

/// Splits on commas not nested inside brackets or strings.
fn split_top_level(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(s[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = s[start..].trim();
    if !last.is_empty() {
        parts.push(s[start..].to_string());
    }
    parts.retain(|p| !p.trim().is_empty());
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const TWO_SUM_STUB: &str =
        "class Solution:\n    def twoSum(self, nums: List[int], target: int) -> List[int]:\n        ";

    const TWO_SUM_BODY: &str = "Given an array of integers `nums` and an integer `target`…\n\
\u{a0}\nExample 1:\n\nInput: nums = [2,7,11,15], target = 9\nOutput: [0,1]\nExplanation: Because nums[0] + nums[1] == 9, we return [0, 1].\n\n\
Example 2:\n\nInput: nums = [3,2,4], target = 6\nOutput: [1,2]\n\n\
Example 3:\n\nInput: nums = [3,3], target = 6\nOutput: [0,1]\n\n\u{a0}\n\
Constraints:\n\n`2 <= nums.length <= 104`\n";

    fn lines(examples: &[&[&str]]) -> Vec<Vec<String>> {
        examples
            .iter()
            .map(|e| e.iter().map(|s| s.to_string()).collect())
            .collect()
    }

    #[test]
    fn two_sum_examples_parse_exactly() {
        let input_lines = lines(&[&["[2,7,11,15]", "9"], &["[3,2,4]", "6"], &["[3,3]", "6"]]);
        let parsed = parse_examples(&input_lines, TWO_SUM_BODY, TWO_SUM_STUB);
        assert_eq!(parsed.report.total, 3);
        assert_eq!(parsed.report.parsed, 3);
        assert!(parsed.report.dropped.is_empty(), "{:?}", parsed.report);
        let expect = [
            (json!([[2, 7, 11, 15], 9]), json!([0, 1])),
            (json!([[3, 2, 4], 6]), json!([1, 2])),
            (json!([[3, 3], 6]), json!([0, 1])),
        ];
        for (case, (input, expected)) in parsed.cases.iter().zip(expect) {
            assert_eq!(case.input, *input.as_array().unwrap());
            assert_eq!(case.expected, expected);
            assert!(!case.hidden);
        }
    }

    #[test]
    fn statement_input_fallback_when_no_structured_lines() {
        let parsed = parse_examples(&[], TWO_SUM_BODY, TWO_SUM_STUB);
        assert_eq!(parsed.report.parsed, 3);
        assert_eq!(parsed.cases[0].input, vec![json!([2, 7, 11, 15]), json!(9)]);
    }

    #[test]
    fn string_and_boolean_answers_parse() {
        let stub = "class Solution:\n    def isPalindrome(self, s: str) -> bool:\n        ";
        let body = "Example 1:\n\nInput: s = \"A man, a plan, a canal: Panama\"\nOutput: true\n\n\
Example 2:\n\nInput: s = \"race a car\"\nOutput: false\n\nConstraints:\n";
        let parsed = parse_examples(
            &lines(&[&["\"A man, a plan, a canal: Panama\""], &["\"race a car\""]]),
            body,
            stub,
        );
        assert_eq!(parsed.report.parsed, 2);
        assert_eq!(
            parsed.cases[0].input,
            vec![json!("A man, a plan, a canal: Panama")]
        );
        assert_eq!(parsed.cases[0].expected, json!(true));
        assert_eq!(parsed.cases[1].expected, json!(false));

        // a string-answer problem
        let stub = "class Solution:\n    def longestCommonPrefix(self, strs: List[str]) -> str:\n";
        let body = "Example 1:\n\nInput: strs = [\"flower\",\"flow\",\"flight\"]\nOutput: \"fl\"\n";
        let parsed = parse_examples(&lines(&[&["[\"flower\",\"flow\",\"flight\"]"]]), body, stub);
        assert_eq!(parsed.report.parsed, 1);
        assert_eq!(parsed.cases[0].expected, json!("fl"));
    }

    #[test]
    fn float_output_parses_as_number() {
        let stub = "class Solution:\n    def findMedianSortedArrays(self, nums1: List[int], nums2: List[int]) -> float:\n";
        let body = "Example 1:\n\nInput: nums1 = [1,3], nums2 = [2]\nOutput: 2.00000\n";
        let parsed = parse_examples(&lines(&[&["[1,3]", "[2]"]]), body, stub);
        assert_eq!(parsed.report.parsed, 1);
        assert_eq!(parsed.cases[0].expected, json!(2.0));
    }

    #[test]
    fn design_problems_are_rejected_here() {
        let stub = "class LRUCache:\n\n    def __init__(self, capacity: int):\n        \n\n    def get(self, key: int) -> int:\n";
        let body =
            "Example 1:\n\nInput\n[\"LRUCache\",\"put\"]\n[[2],[1,1]]\nOutput\n[null, null]\n";
        let parsed = parse_examples(
            &lines(&[&["[\"LRUCache\",\"put\"]", "[[2],[1,1]]"]]),
            body,
            stub,
        );
        assert!(parsed.cases.is_empty());
        assert_eq!(parsed.report.dropped.len(), 1);
        assert!(parsed.report.dropped[0].contains("LRUCache"));
    }

    #[test]
    fn malformed_examples_degrade_per_example_never_guess() {
        // example 2 has an unparseable output; 1 and 3 still parse
        let body = "Example 1:\n\nInput: x = 1\nOutput: 2\n\n\
Example 2:\n\nInput: x = 2\nOutput: see explanation\n\n\
Example 3:\n\nInput: x = 3\nOutput: 6\n\nConstraints:\n";
        let stub = "class Solution:\n    def double(self, x: int) -> int:\n";
        let parsed = parse_examples(&lines(&[&["1"], &["2"], &["3"]]), body, stub);
        assert_eq!(parsed.report.total, 3);
        assert_eq!(parsed.report.parsed, 2);
        assert_eq!(parsed.report.dropped.len(), 1);
        assert!(parsed.report.dropped[0].contains("example 2"));

        // all malformed ⇒ zero cases (the caller demotes to run-only)
        let body = "Example 1:\n\nInput: x = 1\nOutput: impossible\n";
        let parsed = parse_examples(&lines(&[&["1"]]), body, stub);
        assert_eq!(parsed.report.parsed, 0);
        assert!(parsed.cases.is_empty());
    }

    #[test]
    fn arity_mismatch_is_rejected_with_a_reason() {
        // stub takes 2 args, structured input provides 1
        let parsed = parse_examples(
            &lines(&[&["[2,7,11,15]"]]),
            "Example 1:\n\nInput: nums = [2,7,11,15]\nOutput: [0,1]\n",
            TWO_SUM_STUB,
        );
        assert_eq!(parsed.report.parsed, 0);
        assert!(parsed.report.dropped[0].contains("stub takes 2"));
    }

    #[test]
    fn count_mismatch_drops_everything() {
        // 2 structured inputs but 1 statement example — cannot pair
        let parsed = parse_examples(
            &lines(&[&["[1,2]", "3"], &["[4,5]", "9"]]),
            "Example 1:\n\nInput: nums = [1,2], target = 3\nOutput: [0,1]\n",
            TWO_SUM_STUB,
        );
        assert_eq!(parsed.report.parsed, 0);
        assert!(parsed.report.dropped[0].contains("mismatch"));
    }

    #[test]
    fn stub_helpers_parse_real_shapes() {
        assert_eq!(stub_class_name(TWO_SUM_STUB).as_deref(), Some("Solution"));
        assert_eq!(
            stub_class_name("class LRUCache:\n    def __init__…").as_deref(),
            Some("LRUCache")
        );
        assert_eq!(stub_class_name("def solve(a):\n    pass"), None);
        assert_eq!(python_stub_arity(TWO_SUM_STUB), Some(2));
        assert_eq!(
            python_stub_params(TWO_SUM_STUB).unwrap(),
            vec!["nums", "target"]
        );
        // nested annotation brackets don't break the parse
        assert_eq!(
            python_stub_params(
                "class Solution:\n    def f(self, m: dict[str, list[int]], k: int) -> int:\n"
            )
            .unwrap(),
            vec!["m", "k"]
        );
        // top-level function stub (no self)
        assert_eq!(python_stub_arity("def solve(a, b):\n    pass"), Some(2));
    }

    #[test]
    fn input_expression_handles_strings_with_commas_and_equals() {
        assert_eq!(
            parse_input_expression("s = \"a,b = c\", k = 2").unwrap(),
            vec![json!("a,b = c"), json!(2)]
        );
        assert_eq!(
            parse_input_expression("grid = [[1,2],[3,4]]").unwrap(),
            vec![json!([[1, 2], [3, 4]])]
        );
        // no name prefix at all
        assert_eq!(
            parse_input_expression("[1,2]").unwrap(),
            vec![json!([1, 2])]
        );
        // unparseable value ⇒ None
        assert!(parse_input_expression("nums = [1,2,").is_none());
    }
}
