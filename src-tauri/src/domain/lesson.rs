//! Lesson types — one sub-pattern, the atom of the course.
//! A lesson file is Markdown with YAML
//! frontmatter (§7.3): the frontmatter is `LessonFrontmatter`, the body is
//! `explainer_md`. The loader (`services::curriculum`) resolves the
//! frontmatter's `diagram`/`quiz` filename pointers into the actual
//! `DiagramSpec`/`Quiz` and produces the combined `Lesson`, mirroring how
//! `services::catalog` merges a scraped question with its frozen pack.
//!
//! Frontmatter keys are `snake_case` (`trigger_signals`, `worked_example`,
//! `follow_up`) for consistency with every other bundled content schema in
//! this codebase (`Problem`, `TestPack`, `Unit`, `Curriculum`) — the
//! design doc's inline example uses camelCase for a few keys, but that was
//! illustrative, not a hard requirement (§6.1 only fixes the *shape*).

use serde::{Deserialize, Serialize};

use super::diagram::DiagramSpec;
use super::quiz::Quiz;

/// The YAML frontmatter of a `resources/lessons/<unit>/<id>.md` file.
/// `diagram`/`quiz` are filenames resolved relative to the lesson file's
/// own directory.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LessonFrontmatter {
    pub id: String,
    pub unit: String,
    pub subpattern: String,
    #[serde(default)]
    pub trigger_signals: Vec<String>,
    /// The worked-example problem slug (§3.3 item 4).
    pub worked_example: String,
    pub diagram: String,
    pub quiz: String,
    /// Ordered practice slugs, faded -> independent (§3.3 item 6).
    #[serde(default)]
    pub practice: Vec<String>,
    /// Earlier lesson ids this lesson's recap retrieval pulls from
    /// (§3.3 item 7).
    #[serde(default)]
    pub recap: Vec<String>,
    #[serde(default)]
    pub follow_up: Vec<String>,
}

/// A fully loaded lesson (§3.3): the frontmatter plus the resolved
/// `explainer` prose, diagram, and quiz. This is the IPC/TS-mirrored shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub unit: String,
    pub subpattern: String,
    pub explainer_md: String,
    pub trigger_signals: Vec<String>,
    pub worked_example: String,
    pub diagram: DiagramSpec,
    pub quiz: Quiz,
    pub practice: Vec<String>,
    #[serde(default)]
    pub recap: Vec<String>,
    #[serde(default)]
    pub follow_up: Vec<String>,
}

impl Lesson {
    /// Every required part present (§3.3: "all present or the loader
    /// rejects it"). `diagram`/`quiz` validate themselves; this checks the
    /// parts owned directly by the lesson.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("lesson id is empty".into());
        }
        if self.unit.trim().is_empty() {
            return Err(format!("lesson '{}': unit is empty", self.id));
        }
        if self.subpattern.trim().is_empty() {
            return Err(format!("lesson '{}': subpattern is empty", self.id));
        }
        if self.explainer_md.trim().is_empty() {
            return Err(format!("lesson '{}': explainer is empty", self.id));
        }
        if self.trigger_signals.is_empty() {
            return Err(format!(
                "lesson '{}': needs at least one trigger signal",
                self.id
            ));
        }
        if self.worked_example.trim().is_empty() {
            return Err(format!(
                "lesson '{}': worked_example slug is empty",
                self.id
            ));
        }
        if self.practice.is_empty() {
            return Err(format!(
                "lesson '{}': needs at least one practice slug",
                self.id
            ));
        }
        self.diagram
            .validate()
            .map_err(|e| format!("lesson '{}': {e}", self.id))?;
        self.quiz
            .validate()
            .map_err(|e| format!("lesson '{}': {e}", self.id))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::diagram::{DiagramMode, DiagramStep};
    use crate::domain::quiz::{QuizItem, QuizItemType};
    use serde_json::json;

    #[test]
    fn frontmatter_parses_from_yaml() {
        let yaml = r#"
id: 01-hashmap-lookup
unit: arrays-hashing
subpattern: "Hash-map complement lookup"
trigger_signals:
  - "need O(1) membership"
  - "pair that sums to a target"
worked_example: two-sum
diagram: 01-hashmap-lookup.diagram.json
quiz: 01-hashmap-lookup.quiz.json
practice: [contains-duplicate, valid-anagram]
recap: []
follow_up: ["What if the array is sorted?"]
"#;
        let fm: LessonFrontmatter = serde_yaml::from_str(yaml).expect("parses");
        assert_eq!(fm.id, "01-hashmap-lookup");
        assert_eq!(fm.trigger_signals.len(), 2);
        assert_eq!(fm.practice, vec!["contains-duplicate", "valid-anagram"]);
    }

    fn sample_diagram() -> DiagramSpec {
        DiagramSpec {
            id: "d1".into(),
            algorithm: "a".into(),
            for_problem: "two-sum".into(),
            mode: DiagramMode::View,
            steps: vec![DiagramStep {
                state: json!({}),
                caption_md: "s0".into(),
                predict: None,
            }],
            predict_at: vec![0],
        }
    }

    fn sample_quiz() -> Quiz {
        Quiz {
            items: vec![QuizItem {
                id: "q1".into(),
                item_type: QuizItemType::Complexity,
                prompt_md: "p".into(),
                options: vec!["a".into()],
                answer: "a".into(),
                correct_pattern: None,
                explanation_md: "e".into(),
            }],
        }
    }

    fn sample_lesson() -> Lesson {
        Lesson {
            id: "01-hashmap-lookup".into(),
            unit: "arrays-hashing".into(),
            subpattern: "Hash-map complement lookup".into(),
            explainer_md: "A hash map trades space for time.".into(),
            trigger_signals: vec!["pair that sums to a target".into()],
            worked_example: "two-sum".into(),
            diagram: sample_diagram(),
            quiz: sample_quiz(),
            practice: vec!["contains-duplicate".into()],
            recap: vec![],
            follow_up: vec![],
        }
    }

    #[test]
    fn validate_accepts_a_complete_lesson() {
        assert!(sample_lesson().validate().is_ok());
    }

    #[test]
    fn validate_rejects_missing_trigger_signals() {
        let mut l = sample_lesson();
        l.trigger_signals.clear();
        assert!(l.validate().unwrap_err().contains("trigger signal"));
    }

    #[test]
    fn validate_rejects_empty_practice() {
        let mut l = sample_lesson();
        l.practice.clear();
        assert!(l.validate().unwrap_err().contains("practice"));
    }

    #[test]
    fn validate_propagates_diagram_errors() {
        let mut l = sample_lesson();
        l.diagram.predict_at.clear();
        assert!(l.validate().unwrap_err().contains("prediction pause"));
    }

    #[test]
    fn validate_propagates_quiz_errors() {
        let mut l = sample_lesson();
        l.quiz.items.clear();
        assert!(l.validate().unwrap_err().contains("no items"));
    }
}
