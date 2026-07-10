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

/// One frame of the trace: an opaque algorithm-state snapshot (pointer
/// positions, window bounds, hash-map contents, …) plus the caption shown
/// alongside it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiagramStep {
    pub state: Value,
    pub caption_md: String,
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
                },
                DiagramStep {
                    state: json!({}),
                    caption_md: "s1".into(),
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
}
