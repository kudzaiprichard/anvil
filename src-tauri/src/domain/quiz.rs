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
        let mut seen_ids = std::collections::HashSet::new();
        for item in &self.items {
            if item.id.trim().is_empty() {
                return Err("quiz item id is empty".into());
            }
            if !seen_ids.insert(item.id.as_str()) {
                return Err(format!("duplicate quiz item id '{}'", item.id));
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

    /// Grades a set of learner answers against this quiz (Phase 4). Formative
    /// only — the caller records the outcome to feed the review signal, never a
    /// gate. Only the items the learner actually **answered** are graded, so a
    /// quiz split across placements (concept-check mid-lesson, pattern-picker at
    /// the end) can be submitted a section at a time without the other section's
    /// items counting as wrong. Results come back in the quiz's own item order;
    /// an answer for an unknown item id is ignored.
    pub fn grade(&self, answers: &[QuizAnswer]) -> QuizGrade {
        let mut results = Vec::new();
        let mut correct_count = 0u32;
        for item in &self.items {
            let Some(answer) = answers.iter().find(|a| a.item_id == item.id) else {
                continue;
            };
            let correct = answer.selected == item.answer;
            if correct {
                correct_count += 1;
            }
            results.push(QuizItemResult {
                item_id: item.id.clone(),
                item_type: item.item_type,
                correct,
                selected: answer.selected.clone(),
                answer: item.answer.clone(),
                explanation_md: item.explanation_md.clone(),
                correct_pattern: item.correct_pattern.clone(),
            });
        }
        QuizGrade {
            correct_count,
            total: results.len() as u32,
            results,
        }
    }
}

/// One learner answer to a quiz item — the item's id and the option text the
/// learner selected (`submit_quiz` payload).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizAnswer {
    pub item_id: String,
    pub selected: String,
}

/// The graded outcome of one quiz item (`submit_quiz` result). `answer` and
/// `explanation_md` are echoed back so the runner reveals the trigger after the
/// learner commits — a pattern-picker additionally surfaces `correct_pattern`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizItemResult {
    pub item_id: String,
    #[serde(rename = "type")]
    pub item_type: QuizItemType,
    pub correct: bool,
    pub selected: String,
    /// The correct option text.
    pub answer: String,
    pub explanation_md: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_pattern: Option<String>,
}

/// The full graded submission returned by `submit_quiz` — never blocks
/// progression (LESSON_COURSE_DESIGN.md §3.4: quizzes are formative).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizGrade {
    pub correct_count: u32,
    pub total: u32,
    pub results: Vec<QuizItemResult>,
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

    #[test]
    fn validate_rejects_duplicate_item_ids() {
        let quiz = Quiz {
            items: vec![
                item("q1", QuizItemType::Complexity, &["a", "b"], "a"),
                item("q1", QuizItemType::Complexity, &["a", "b"], "b"),
            ],
        };
        assert!(quiz.validate().unwrap_err().contains("duplicate quiz item id"));
    }

    fn sample_quiz() -> Quiz {
        let mut picker = item("q2", QuizItemType::PatternPicker, &["hash", "sort"], "hash");
        picker.correct_pattern = Some("arrays-hashing".into());
        Quiz {
            items: vec![
                item("q1", QuizItemType::ConceptCheck, &["value", "index"], "value"),
                picker,
            ],
        }
    }

    #[test]
    fn grade_scores_selected_answers_in_item_order() {
        let quiz = sample_quiz();
        let grade = quiz.grade(&[
            QuizAnswer { item_id: "q2".into(), selected: "hash".into() },
            QuizAnswer { item_id: "q1".into(), selected: "index".into() },
        ]);
        assert_eq!(grade.total, 2);
        assert_eq!(grade.correct_count, 1);
        // Results follow item order, not answer order.
        assert_eq!(grade.results[0].item_id, "q1");
        assert!(!grade.results[0].correct);
        assert_eq!(grade.results[1].item_id, "q2");
        assert!(grade.results[1].correct);
        // Pattern-picker echoes the pattern; concept-check does not.
        assert_eq!(grade.results[1].correct_pattern.as_deref(), Some("arrays-hashing"));
        assert!(grade.results[0].correct_pattern.is_none());
    }

    #[test]
    fn grade_only_scores_answered_items() {
        let quiz = sample_quiz();
        // Only q1 answered; q2 left blank — a section submitted on its own.
        let grade = quiz.grade(&[QuizAnswer {
            item_id: "q1".into(),
            selected: "value".into(),
        }]);
        assert_eq!(grade.total, 1);
        assert_eq!(grade.correct_count, 1);
        assert_eq!(grade.results.len(), 1);
        assert_eq!(grade.results[0].item_id, "q1");
    }

    #[test]
    fn grade_ignores_answers_for_unknown_items() {
        let quiz = sample_quiz();
        let grade = quiz.grade(&[QuizAnswer {
            item_id: "ghost".into(),
            selected: "whatever".into(),
        }]);
        assert_eq!(grade.correct_count, 0);
        assert_eq!(grade.total, 0);
        assert!(grade.results.is_empty());
    }

    #[test]
    fn quiz_grade_round_trips_camel_case() {
        let grade = sample_quiz().grade(&[QuizAnswer {
            item_id: "q2".into(),
            selected: "hash".into(),
        }]);
        let value = serde_json::to_value(&grade).unwrap();
        assert_eq!(value["correctCount"], json!(1));
        assert_eq!(value["total"], json!(1));
        // Only the answered item (q2) is graded.
        assert_eq!(value["results"][0]["itemId"], json!("q2"));
        assert_eq!(value["results"][0]["type"], json!("pattern-picker"));
        assert_eq!(value["results"][0]["correctPattern"], json!("arrays-hashing"));
        let back: QuizGrade = serde_json::from_value(value).unwrap();
        assert_eq!(back, grade);
    }
}
