//! Quiz types (LESSON_COURSE_DESIGN.md §3.4, §13.3) — formative checks
//! only, never a gate. Three item types share one shape: `concept-check`
//! and `complexity` grade against `answer`; `pattern-picker` is prompt-only
//! (unlabeled) and additionally carries `correct_pattern`, the transfer
//! skill this course trains. Field names mirror `src/lib/types.ts`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuizItemType {
    ConceptCheck,
    PatternPicker,
    Complexity,
}

/// One quiz item (§7.4 example). `answer` must equal one of `options`
/// (enforced by the loader, mirrors `tools/build_curriculum.py --check`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuizItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: QuizItemType,
    pub prompt_md: String,
    pub options: Vec<String>,
    pub answer: String,
    /// The unit/pattern id a `pattern-picker` item is testing recognition
    /// of; absent for `concept-check`/`complexity` items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct_pattern: Option<String>,
    pub explanation_md: String,
}

/// One lesson's quiz file (§7.4): `01-hashmap-lookup.quiz.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quiz {
    pub items: Vec<QuizItem>,
}

impl Quiz {
    /// Structural rules: at least one item, `answer` in `options`,
    /// `pattern-picker` items carry `correct_pattern`.
    pub fn validate(&self) -> Result<(), String> {
        if self.items.is_empty() {
            return Err("quiz has no items".into());
        }
        for item in &self.items {
            if item.id.trim().is_empty() {
                return Err("quiz item id is empty".into());
            }
            if !item.options.iter().any(|o| o == &item.answer) {
                return Err(format!(
                    "quiz item '{}': answer '{}' is not among options",
                    item.id, item.answer
                ));
            }
            if item.item_type == QuizItemType::PatternPicker && item.correct_pattern.is_none() {
                return Err(format!(
                    "quiz item '{}': pattern-picker items require correct_pattern",
                    item.id
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
    fn quiz_round_trips_the_design_doc_example() {
        round_trip::<Quiz>(json!({ "items": [
            { "id": "q1", "type": "pattern-picker",
              "prompt_md": "Given an unsorted array, find if any two numbers sum to K.",
              "options": ["Sort + two pointers", "Hash-map complement", "Sliding window"],
              "answer": "Hash-map complement", "correct_pattern": "arrays-hashing",
              "explanation_md": "Unsorted + pair-sum + O(n) -> complement set." },
            { "id": "q2", "type": "complexity",
              "prompt_md": "Time complexity of the one-pass hash-map solution?",
              "options": ["O(n)", "O(n log n)", "O(n^2)"], "answer": "O(n)",
              "explanation_md": "One pass, O(1) lookups." }
        ]}));
    }

    #[test]
    fn correct_pattern_is_omitted_when_absent() {
        let item: QuizItem = serde_json::from_value(json!({
            "id": "q2", "type": "complexity", "prompt_md": "p",
            "options": ["a", "b"], "answer": "a", "explanation_md": "e"
        }))
        .unwrap();
        let v = serde_json::to_value(&item).unwrap();
        assert!(v.get("correct_pattern").is_none());
    }

    fn item(id: &str, ty: QuizItemType, options: &[&str], answer: &str) -> QuizItem {
        QuizItem {
            id: id.into(),
            item_type: ty,
            prompt_md: "p".into(),
            options: options.iter().map(|s| s.to_string()).collect(),
            answer: answer.into(),
            correct_pattern: None,
            explanation_md: "e".into(),
        }
    }

    #[test]
    fn validate_rejects_answer_not_in_options() {
        let quiz = Quiz {
            items: vec![item("q1", QuizItemType::Complexity, &["a", "b"], "c")],
        };
        assert!(quiz.validate().unwrap_err().contains("not among options"));
    }

    #[test]
    fn validate_requires_correct_pattern_on_pattern_picker() {
        let quiz = Quiz {
            items: vec![item("q1", QuizItemType::PatternPicker, &["a", "b"], "a")],
        };
        assert!(quiz.validate().unwrap_err().contains("pattern-picker"));

        let mut with_pattern = quiz;
        with_pattern.items[0].correct_pattern = Some("arrays-hashing".into());
        assert!(with_pattern.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_quiz() {
        assert!(Quiz { items: vec![] }
            .validate()
            .unwrap_err()
            .contains("no items"));
    }
}
