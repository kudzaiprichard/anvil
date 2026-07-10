//! Curriculum types — the one implicit course (LESSON_COURSE_DESIGN.md
//! §3.1). `curriculum.json` is a single file: ordered stages, the unit
//! list, the prerequisite DAG, and global gate defaults. Field names mirror
//! `src/lib/types.ts` exactly.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::unit::GateConfig;

/// One visible "level" (§4): an ordered group of unit ids.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurriculumStage {
    pub id: String,
    pub title: String,
    pub units: Vec<String>,
}

/// The whole track (§7.1 example). Loaded from
/// `resources/curriculum/curriculum.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Curriculum {
    pub id: String,
    pub stages: Vec<CurriculumStage>,
    /// `unitId -> [prereq unitId]`, the prerequisite DAG driving unlocking
    /// (BLUEPRINT.md §4).
    #[serde(default)]
    pub prereqs: HashMap<String, Vec<String>>,
    /// Fallback gate knobs a unit may override in its own manifest.
    pub gate_defaults: GateConfig,
}

impl Curriculum {
    /// Every unit id referenced by a stage, in stage order (may repeat if a
    /// malformed file lists one twice — the loader's cross-check catches
    /// that against the actually-discovered unit files).
    pub fn unit_ids(&self) -> Vec<&str> {
        self.stages
            .iter()
            .flat_map(|s| s.units.iter().map(String::as_str))
            .collect()
    }

    /// Structural rules enforced by the loader: non-empty identity, at
    /// least one stage, every stage non-empty, every prereq/prereq-target
    /// pair references a unit id actually listed in some stage.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("curriculum id is empty".into());
        }
        if self.stages.is_empty() {
            return Err("curriculum has no stages".into());
        }
        let known: std::collections::HashSet<&str> = self.unit_ids().into_iter().collect();
        for stage in &self.stages {
            if stage.id.trim().is_empty() || stage.title.trim().is_empty() {
                return Err("stage id/title is empty".into());
            }
            if stage.units.is_empty() {
                return Err(format!("stage '{}' has no units", stage.id));
            }
        }
        for (unit, deps) in &self.prereqs {
            if !known.contains(unit.as_str()) {
                return Err(format!("prereqs: unknown unit '{unit}'"));
            }
            for dep in deps {
                if !known.contains(dep.as_str()) {
                    return Err(format!("prereqs: unit '{unit}' has unknown prereq '{dep}'"));
                }
            }
        }
        detect_cycle(&self.prereqs)?;
        Ok(())
    }
}

/// DFS cycle detection over the prereq adjacency map (build_curriculum.py
/// §8 "prereq graph is a DAG (no cycles)", mirrored here for fail-closed
/// startup validation).
fn detect_cycle(prereqs: &HashMap<String, Vec<String>>) -> Result<(), String> {
    #[derive(PartialEq, Clone, Copy)]
    enum Mark {
        Visiting,
        Done,
    }
    fn visit<'a>(
        node: &'a str,
        prereqs: &'a HashMap<String, Vec<String>>,
        marks: &mut HashMap<&'a str, Mark>,
        stack: &mut Vec<&'a str>,
    ) -> Result<(), String> {
        match marks.get(node) {
            Some(Mark::Done) => return Ok(()),
            Some(Mark::Visiting) => {
                stack.push(node);
                return Err(format!("prereq cycle: {}", stack.join(" -> ")));
            }
            None => {}
        }
        marks.insert(node, Mark::Visiting);
        stack.push(node);
        if let Some(deps) = prereqs.get(node) {
            for dep in deps {
                visit(dep, prereqs, marks, stack)?;
            }
        }
        stack.pop();
        marks.insert(node, Mark::Done);
        Ok(())
    }

    let mut marks = HashMap::new();
    for node in prereqs.keys() {
        let mut stack = Vec::new();
        visit(node.as_str(), prereqs, &mut marks, &mut stack)?;
    }
    Ok(())
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
    fn curriculum_round_trips_the_design_doc_example() {
        round_trip::<Curriculum>(json!({
            "id": "dsa-track",
            "stages": [
                { "id": "s1", "title": "Array Fundamentals",
                  "units": ["arrays-hashing", "two-pointers", "sliding-window"] }
            ],
            "prereqs": {
                "two-pointers": ["arrays-hashing"],
                "sliding-window": ["two-pointers"]
            },
            "gate_defaults": { "pass_count": 4, "require_novel": true, "timer_target_min": 25, "threshold_pct": 80 }
        }));
    }

    fn sample() -> Curriculum {
        Curriculum {
            id: "dsa-track".into(),
            stages: vec![CurriculumStage {
                id: "s1".into(),
                title: "Array Fundamentals".into(),
                units: vec!["a".into(), "b".into(), "c".into()],
            }],
            prereqs: HashMap::from([
                ("b".to_string(), vec!["a".to_string()]),
                ("c".to_string(), vec!["b".to_string()]),
            ]),
            gate_defaults: GateConfig {
                pass_count: 4,
                require_novel: true,
                timer_target_min: 25,
                threshold_pct: 80,
            },
        }
    }

    #[test]
    fn validate_accepts_a_dag() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn validate_rejects_a_cycle() {
        let mut c = sample();
        c.prereqs.insert("a".into(), vec!["c".into()]);
        let err = c.validate().unwrap_err();
        assert!(err.contains("cycle"), "{err}");
    }

    #[test]
    fn validate_rejects_unknown_unit_in_prereqs() {
        let mut c = sample();
        c.prereqs.insert("ghost".into(), vec!["a".into()]);
        assert!(c.validate().unwrap_err().contains("unknown unit"));
    }

    #[test]
    fn validate_rejects_empty_stage() {
        let mut c = sample();
        c.stages[0].units.clear();
        assert!(c.validate().unwrap_err().contains("no units"));
    }
}
