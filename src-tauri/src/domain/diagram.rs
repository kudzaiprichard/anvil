//! Prediction-diagram types (LESSON_COURSE_DESIGN.md §3.5, §13.4). The
//! renderer/animator is engine (Phase 5); the steps/trace are data,
//! precomputed offline — no server, no runtime execution. Field names
//! mirror `src/lib/types.ts`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagramMode {
    View,
    Perform,
}

/// One graded choice offered at a prediction pause.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramChoice {
    pub id: String,
    pub label_md: String,
}

/// The graded "what happens next?" turn attached to a prediction-pause step
/// (§13.4). The renderer reveals `explanation_md` after the learner commits.
/// Optional: a pause step without a `predict` block degrades to a
/// think-then-reveal prompt (the caption carries the question). When present
/// in `perform` mode it is the learner's step graded against ground truth
/// (COURSE_BLUEPRINT.md §7, "perform the algorithm yourself").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramPredict {
    pub prompt_md: String,
    pub choices: Vec<DiagramChoice>,
    /// The `id` of the correct choice — engine ground truth for the step.
    pub answer: String,
    pub explanation_md: String,
}

impl DiagramPredict {
    fn validate(&self, where_: &str) -> Result<(), String> {
        if self.choices.len() < 2 {
            return Err(format!("{where_}: a prediction needs at least two choices"));
        }
        if !self.choices.iter().any(|c| c.id == self.answer) {
            return Err(format!(
                "{where_}: prediction answer '{}' is not one of its choices",
                self.answer
            ));
        }
        Ok(())
    }
}

/// One frame of the trace: an opaque algorithm-state snapshot (pointer
/// positions, window bounds, hash-map contents, …) plus the caption shown
/// alongside it. A frame listed in `predict_at` may carry a graded `predict`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramStep {
    pub state: Value,
    pub caption_md: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predict: Option<DiagramPredict>,
}

/// One lesson's diagram spec (§7.5 example):
/// `01-hashmap-lookup.diagram.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramSpec {
    pub id: String,
    pub algorithm: String,
    /// The worked-example problem slug this diagram is keyed to.
    pub for_problem: String,
    pub mode: DiagramMode,
    pub steps: Vec<DiagramStep>,
    /// Step indices where playback pauses to ask "what happens next?"
    /// (§13.4 — required: at least one).
    pub predict_at: Vec<usize>,
}

impl DiagramSpec {
    /// Structural rules: at least one step, at least one prediction pause,
    /// every `predict_at` index is a valid step index.
    pub fn validate(&self) -> Result<(), String> {
        if self.steps.is_empty() {
            return Err(format!("diagram '{}': has no steps", self.id));
        }
        if self.predict_at.is_empty() {
            return Err(format!(
                "diagram '{}': needs at least one prediction pause",
                self.id
            ));
        }
        for &idx in &self.predict_at {
            if idx >= self.steps.len() {
                return Err(format!(
                    "diagram '{}': predict_at index {idx} out of range (has {} steps)",
                    self.id,
                    self.steps.len()
                ));
            }
        }
        // A graded prediction, wherever it appears, must be internally
        // consistent (answer ∈ choices, ≥2 choices).
        for (i, step) in self.steps.iter().enumerate() {
            if let Some(predict) = &step.predict {
                predict.validate(&format!("diagram '{}' step {i}", self.id))?;
            }
        }
        Ok(())
    }
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
    fn diagram_round_trips_the_design_doc_example() {
        round_trip::<DiagramSpec>(json!({
            "id": "hashmap-lookup", "algorithm": "two-sum-hashmap", "for_problem": "two-sum",
            "mode": "view",
            "steps": [
                { "state": { "i": 0, "seen": {}, "num": 2, "need": 7 }, "caption_md": "Store 2, look for 7." },
                { "state": { "i": 1, "seen": {"2": 0}, "num": 7, "need": 2 }, "caption_md": "7 needs 2 -- it's in the map." }
            ],
            "predict_at": [1]
        }));
    }

    fn sample() -> DiagramSpec {
        DiagramSpec {
            id: "d1".into(),
            algorithm: "a".into(),
            for_problem: "two-sum".into(),
            mode: DiagramMode::View,
            steps: vec![
                DiagramStep {
                    state: json!({}),
                    caption_md: "s0".into(),
                    predict: None,
                },
                DiagramStep {
                    state: json!({}),
                    caption_md: "s1".into(),
                    predict: None,
                },
            ],
            predict_at: vec![1],
        }
    }

    #[test]
    fn validate_accepts_a_valid_spec() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn validate_rejects_out_of_range_predict_at() {
        let mut d = sample();
        d.predict_at = vec![5];
        assert!(d.validate().unwrap_err().contains("out of range"));
    }

    #[test]
    fn validate_rejects_no_prediction_pause() {
        let mut d = sample();
        d.predict_at = vec![];
        assert!(d.validate().unwrap_err().contains("prediction pause"));
    }

    #[test]
    fn validate_rejects_no_steps() {
        let mut d = sample();
        d.steps = vec![];
        d.predict_at = vec![0];
        assert!(d.validate().unwrap_err().contains("no steps"));
    }

    #[test]
    fn graded_prediction_round_trips_and_is_optional() {
        // Absent by default: a step without `predict` serializes without the key.
        let v = serde_json::to_value(DiagramStep {
            state: json!({}),
            caption_md: "c".into(),
            predict: None,
        })
        .unwrap();
        assert!(v.get("predict").is_none());

        // Present: full round-trip through the graded-choice shape.
        round_trip::<DiagramStep>(json!({
            "state": { "i": 1 },
            "caption_md": "What happens next?",
            "predict": {
                "prompt_md": "Pick the next move.",
                "choices": [
                    { "id": "a", "label_md": "Look up the complement." },
                    { "id": "b", "label_md": "Scan every earlier value." }
                ],
                "answer": "a",
                "explanation_md": "The map already holds it — one O(1) lookup."
            }
        }));
    }

    #[test]
    fn validate_rejects_inconsistent_prediction() {
        let mut d = sample();
        d.steps[1].predict = Some(DiagramPredict {
            prompt_md: "p".into(),
            choices: vec![DiagramChoice {
                id: "a".into(),
                label_md: "only one".into(),
            }],
            answer: "a".into(),
            explanation_md: "e".into(),
        });
        assert!(d.validate().unwrap_err().contains("at least two choices"));

        d.steps[1].predict = Some(DiagramPredict {
            prompt_md: "p".into(),
            choices: vec![
                DiagramChoice {
                    id: "a".into(),
                    label_md: "x".into(),
                },
                DiagramChoice {
                    id: "b".into(),
                    label_md: "y".into(),
                },
            ],
            answer: "z".into(),
            explanation_md: "e".into(),
        });
        assert!(d.validate().unwrap_err().contains("not one of its choices"));
    }
}
