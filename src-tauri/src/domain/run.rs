//! Run types — the IPC contract for the workspace Run/Submit flow. Result
//! fields are camelCase (`runtimeMs`, `memoryMb`) unlike problem fields;
//! that mixed convention is what `src/lib/types.ts` ships and is contract-
//! tested below. `CaseResult` display fields are pre-formatted strings and
//! must be omitted (not null) for hidden cases — the privacy invariant.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Javascript,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRequest {
    pub id: String,
    pub language: Language,
    pub code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Pass,
    Fail,
    Error,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseResult {
    /// 1-based case number within the executed set.
    pub index: u32,
    pub hidden: bool,
    pub passed: bool,
    /// Display strings; omitted for hidden cases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub status: RunStatus,
    pub cases: Vec<CaseResult>,
    pub passed: u32,
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<f64>,
    /// stderr / traceback when status is "error" or "timeout".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn round_trip<T: Serialize + for<'de> Deserialize<'de>>(value: serde_json::Value) {
        let parsed: T = serde_json::from_value(value.clone()).expect("deserialize");
        assert_eq!(serde_json::to_value(&parsed).expect("serialize"), value);
    }

    #[test]
    fn language_and_status_strings_match_types_ts() {
        assert_eq!(
            serde_json::to_value(Language::Python).unwrap(),
            json!("python")
        );
        assert_eq!(
            serde_json::to_value(Language::Javascript).unwrap(),
            json!("javascript")
        );
        assert_eq!(
            serde_json::to_value(RunStatus::Pass).unwrap(),
            json!("pass")
        );
        assert_eq!(
            serde_json::to_value(RunStatus::Timeout).unwrap(),
            json!("timeout")
        );
    }

    #[test]
    fn run_request_round_trips() {
        round_trip::<RunRequest>(json!({
            "id": "pair-with-target-sum",
            "language": "python",
            "code": "def solve(nums, target):\n    return []"
        }));
    }

    #[test]
    fn run_result_uses_camel_case_and_omits_absent_options() {
        round_trip::<RunResult>(json!({
            "status": "fail",
            "cases": [
                { "index": 1, "hidden": false, "passed": true,
                  "input": "nums=[2,7], target=9", "output": "[0, 1]", "expected": "[0, 1]" },
                { "index": 2, "hidden": true, "passed": false }
            ],
            "passed": 1,
            "total": 2,
            "runtimeMs": 12
        }));

        let timeout = RunResult {
            status: RunStatus::Timeout,
            cases: vec![],
            passed: 0,
            total: 3,
            runtime_ms: None,
            memory_mb: None,
            error: Some("Time limit exceeded — execution stopped after 3000 ms.".into()),
        };
        let v = serde_json::to_value(&timeout).unwrap();
        assert!(v.get("runtimeMs").is_none());
        assert!(v.get("memoryMb").is_none());
        assert_eq!(
            v["error"],
            "Time limit exceeded — execution stopped after 3000 ms."
        );
    }
}
