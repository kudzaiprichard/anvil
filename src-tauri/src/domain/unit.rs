//! Unit types — the mastery-gate boundary (LESSON_COURSE_DESIGN.md §3.2).
//! One unit = one concept (e.g. "Arrays & Hashing"); it owns an ordered
//! lesson list and a problem pool split into worked/guided/gate roles.
//! Field names mirror `src/lib/types.ts` exactly, same convention as
//! `domain::pack`/`domain::preset`.

use serde::{Deserialize, Serialize};

/// A problem's role within its unit (§5): worked examples are narrated in
/// the lesson, guided practice ramps difficulty, gate problems are fresh and
/// never shown before the mastery check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemRole {
    Worked,
    Guided,
    Gate,
}

/// Coarse difficulty band within a role, for ramping (§13.1 "faded →
/// independent").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProblemTier {
    Intro,
    Core,
    Stretch,
}

/// One LeetCode slug's place in the unit's problem pool. The statement text
/// is never shipped — only the slug, role, tier, and novelty flag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitProblem {
    pub slug: String,
    pub role: ProblemRole,
    pub tier: ProblemTier,
    pub novel: bool,
}

/// Mastery-gate enforcement knobs (§3.6, §6): fresh problems only, hints
/// off, solution hidden, soft timer, pass = `pass_count` incl. `require_novel`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GateConfig {
    pub pass_count: u32,
    pub require_novel: bool,
    pub timer_target_min: u32,
    pub threshold_pct: u32,
}

/// A unit manifest (§3.2, §7.2 example). Loaded from
/// `resources/curriculum/units/<id>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub stage: String,
    pub title: String,
    #[serde(default)]
    pub prereqs: Vec<String>,
    /// Lesson ids in order; empty until Phase 2 authors lesson content.
    #[serde(default)]
    pub lessons: Vec<String>,
    pub problems: Vec<UnitProblem>,
    pub gate: GateConfig,
    /// Unit ids whose pattern this unit's practice must resurface (§3.2).
    #[serde(default)]
    pub spiral: Vec<String>,
}

impl Unit {
    /// Structural rules enforced by the loader (fail-closed, mirrors
    /// `Preset::validate`): non-empty identity, at least one problem, and
    /// every problem slug used at most once.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("unit id is empty".into());
        }
        if self.stage.trim().is_empty() {
            return Err(format!("unit '{}': stage is empty", self.id));
        }
        if self.title.trim().is_empty() {
            return Err(format!("unit '{}': title is empty", self.id));
        }
        if self.problems.is_empty() {
            return Err(format!("unit '{}': has no problems", self.id));
        }
        let mut seen = std::collections::HashSet::new();
        for p in &self.problems {
            if p.slug.trim().is_empty() {
                return Err(format!("unit '{}': empty problem slug", self.id));
            }
            if !seen.insert(p.slug.as_str()) {
                return Err(format!(
                    "unit '{}': duplicate problem slug '{}'",
                    self.id, p.slug
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
    fn enums_use_lowercase_json_values() {
        assert_eq!(
            serde_json::to_value(ProblemRole::Worked).unwrap(),
            json!("worked")
        );
        assert_eq!(
            serde_json::to_value(ProblemRole::Guided).unwrap(),
            json!("guided")
        );
        assert_eq!(
            serde_json::to_value(ProblemRole::Gate).unwrap(),
            json!("gate")
        );
        assert_eq!(
            serde_json::to_value(ProblemTier::Intro).unwrap(),
            json!("intro")
        );
        assert_eq!(
            serde_json::to_value(ProblemTier::Core).unwrap(),
            json!("core")
        );
        assert_eq!(
            serde_json::to_value(ProblemTier::Stretch).unwrap(),
            json!("stretch")
        );
    }

    #[test]
    fn unit_round_trips_the_design_doc_example() {
        round_trip::<Unit>(json!({
            "id": "arrays-hashing", "stage": "s1", "title": "Arrays & Hashing",
            "prereqs": [], "lessons": ["01-hashmap-lookup", "02-frequency-count", "03-prefix-sum"],
            "problems": [
                { "slug": "two-sum", "role": "worked", "tier": "intro", "novel": false },
                { "slug": "top-k-frequent-elements", "role": "guided", "tier": "core", "novel": false },
                { "slug": "subarray-sum-equals-k", "role": "gate", "tier": "stretch", "novel": true }
            ],
            "gate": { "pass_count": 4, "require_novel": true, "timer_target_min": 25, "threshold_pct": 80 },
            "spiral": []
        }));
    }

    #[test]
    fn validate_rejects_empty_and_duplicate_slugs() {
        let mut u = Unit {
            id: "x".into(),
            stage: "s1".into(),
            title: "X".into(),
            prereqs: vec![],
            lessons: vec![],
            problems: vec![
                UnitProblem {
                    slug: "a".into(),
                    role: ProblemRole::Worked,
                    tier: ProblemTier::Intro,
                    novel: false,
                },
                UnitProblem {
                    slug: "a".into(),
                    role: ProblemRole::Guided,
                    tier: ProblemTier::Core,
                    novel: false,
                },
            ],
            gate: GateConfig {
                pass_count: 1,
                require_novel: false,
                timer_target_min: 10,
                threshold_pct: 80,
            },
            spiral: vec![],
        };
        assert!(u.validate().unwrap_err().contains("duplicate"));
        u.problems.truncate(1);
        assert!(u.validate().is_ok());
        u.problems.clear();
        assert!(u.validate().unwrap_err().contains("no problems"));
    }
}
