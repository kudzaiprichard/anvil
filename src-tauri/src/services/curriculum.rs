//! Curriculum + lesson loading: glob-
//! discovers `resources/curriculum/**` + `resources/lessons/**`, parses,
//! and validates at startup — fail-closed, mirroring `preset_store.rs`. A
//! malformed curriculum/unit/lesson/quiz/diagram file, a dangling
//! stage->unit/unit->lesson reference, or a unit problem slug with no
//! frozen pack all abort startup with a precise, file-naming error rather
//! than loading partial content.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::domain::curriculum::{Capstone, Curriculum};
use crate::domain::diagram::DiagramSpec;
use crate::domain::lesson::{Lesson, LessonFrontmatter};
use crate::domain::quiz::{Quiz, QuizItemType};
use crate::domain::unit::Unit;
use crate::error::{AppError, AppResult};
use crate::services::pack_store::PackStore;

#[derive(Debug)]
pub struct CurriculumStore {
    curriculum: Curriculum,
    units: HashMap<String, Unit>,
    lessons: HashMap<String, Lesson>,
    /// The interleaved, cross-unit pattern-picker pool (Phase 4):
    /// unlabeled "which pattern?" prompts drawn
    /// from across the stage, used for spaced/interleaved formative retrieval.
    /// Empty when no `curriculum/pattern-pool.json` ships (optional content,
    /// like lessons).
    pattern_pool: Quiz,
}

impl CurriculumStore {
    /// Loads and fully validates the curriculum. `packs` backs the
    /// "every referenced slug has a frozen pack" check (§8).
    pub fn load(resources_dir: &Path, packs: &PackStore) -> AppResult<Self> {
        let curriculum_dir = resources_dir.join("curriculum");
        let curriculum = load_curriculum(&curriculum_dir.join("curriculum.json"))?;
        let units = load_units(&curriculum_dir.join("units"))?;
        let lessons = load_lessons(&resources_dir.join("lessons"))?;
        let pattern_pool = load_pattern_pool(&curriculum_dir.join("pattern-pool.json"))?;

        for id in curriculum.unit_ids() {
            if !units.contains_key(id) {
                return Err(AppError::Validation(format!(
                    "curriculum.json: stage references unknown unit '{id}'"
                )));
            }
        }
        for unit in units.values() {
            for lesson_id in &unit.lessons {
                if !lessons.contains_key(lesson_id) {
                    return Err(AppError::Validation(format!(
                        "unit '{}' references unknown lesson '{lesson_id}'",
                        unit.id
                    )));
                }
            }
            for p in &unit.problems {
                if packs.get(&p.slug).is_none() {
                    return Err(AppError::Validation(format!(
                        "unit '{}': problem slug '{}' has no frozen pack",
                        unit.id, p.slug
                    )));
                }
            }
        }
        for lesson in lessons.values() {
            if !units.contains_key(&lesson.unit) {
                return Err(AppError::Validation(format!(
                    "lesson '{}' references unknown unit '{}'",
                    lesson.id, lesson.unit
                )));
            }
        }

        // Spiral-reuse enforcement. Two rules, fail-closed:
        //   (a) every unit's `spiral` entries name a real unit that is an
        //       *ancestor* (transitive prereq) — you only resurface a pattern the
        //       learner has already been taught; and
        //   (b) *coverage*: every unit that some other unit depends on must be
        //       resurfaced by at least one later unit's spiral, so no taught,
        //       depended-on pattern silently drops out of rotation.
        let ancestors = transitive_prereqs(&curriculum);
        let mut spiralled: HashSet<&str> = HashSet::new();
        for unit in units.values() {
            let unit_ancestors = ancestors.get(unit.id.as_str());
            for s in &unit.spiral {
                if !units.contains_key(s) {
                    return Err(AppError::Validation(format!(
                        "unit '{}': spiral references unknown unit '{s}'",
                        unit.id
                    )));
                }
                if !unit_ancestors.is_some_and(|a| a.contains(s.as_str())) {
                    return Err(AppError::Validation(format!(
                        "unit '{}': spiral entry '{s}' is not a prerequisite (can only \
                         resurface an already-taught pattern)",
                        unit.id
                    )));
                }
                spiralled.insert(s.as_str());
            }
        }
        let mut depended_on: HashSet<&str> = HashSet::new();
        for deps in curriculum.prereqs.values() {
            for d in deps {
                depended_on.insert(d.as_str());
            }
        }
        for dep in &depended_on {
            if !spiralled.contains(dep) {
                return Err(AppError::Validation(format!(
                    "spiral coverage: unit '{dep}' is a prerequisite of another unit but is \
                     never resurfaced by any unit's spiral"
                )));
            }
        }

        // The Stage-7 capstone (Phase 7): every problem must have a frozen pack
        // and name a real unit — the unit is used server-side for scoring and is
        // never sent to the workspace (unlabeled exam, §4).
        if let Some(cap) = &curriculum.capstone {
            for p in &cap.problems {
                if packs.get(&p.slug).is_none() {
                    return Err(AppError::Validation(format!(
                        "capstone: problem slug '{}' has no frozen pack",
                        p.slug
                    )));
                }
                if !units.contains_key(&p.unit) {
                    return Err(AppError::Validation(format!(
                        "capstone: problem '{}' names unknown unit '{}'",
                        p.slug, p.unit
                    )));
                }
            }
        }

        // Every pattern-pool item's `correct_pattern` must name a real unit —
        // the pool trains recognition *of the units in this course*.
        for item in &pattern_pool.items {
            if let Some(pattern) = &item.correct_pattern {
                if !units.contains_key(pattern) {
                    return Err(AppError::Validation(format!(
                        "pattern-pool item '{}': correct_pattern '{pattern}' is not a known unit",
                        item.id
                    )));
                }
            }
        }

        log::info!(
            "curriculum: {} units, {} lessons, {} pattern-pool item(s) loaded from {}",
            units.len(),
            lessons.len(),
            pattern_pool.items.len(),
            resources_dir.display()
        );

        Ok(Self {
            curriculum,
            units,
            lessons,
            pattern_pool,
        })
    }

    pub fn curriculum(&self) -> &Curriculum {
        &self.curriculum
    }

    pub fn get_unit(&self, id: &str) -> Option<&Unit> {
        self.units.get(id)
    }

    pub fn get_lesson(&self, id: &str) -> Option<&Lesson> {
        self.lessons.get(id)
    }

    /// A lesson's quiz (Phase 4 `get_quiz`) — the formative concept-check +
    /// pattern-picker items authored alongside the lesson.
    pub fn get_quiz(&self, lesson_id: &str) -> Option<&Quiz> {
        self.lessons.get(lesson_id).map(|l| &l.quiz)
    }

    /// The interleaved cross-unit pattern-picker pool (may be empty).
    pub fn pattern_pool(&self) -> &Quiz {
        &self.pattern_pool
    }

    /// The Stage-7 mixed capstone, if this curriculum ships one.
    pub fn capstone(&self) -> Option<&Capstone> {
        self.curriculum.capstone.as_ref()
    }

    /// All unit ids, in stage order (curriculum spine).
    pub fn unit_ids(&self) -> Vec<&str> {
        self.curriculum.unit_ids()
    }

    /// The transitive prerequisites (ancestors) of `unit_id` — every unit that
    /// must be mastered, directly or indirectly, before it. Used by
    /// partial-prereq-credit review (a solve in a unit credits its ancestors).
    pub fn ancestors_of(&self, unit_id: &str) -> HashSet<String> {
        transitive_prereqs(&self.curriculum)
            .get(unit_id)
            .map(|set| set.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }

    /// Units that legitimately cite another unit's problem as an illustrative
    /// example — Big-O borrows problems from every pattern to teach complexity
    /// classes; Design/OOD combines an earlier pattern's structure into a
    /// design problem (e.g. LRU reuses a hash-map + linked-list slug) — but
    /// neither one "owns" that problem's actual solving pattern. When a slug
    /// is declared in more than one unit's `problems[]` (the full ladder has a
    /// dozen such cases), [`unit_of_problem`] prefers whichever match is *not*
    /// one of these, so a review card is tagged by its real technique instead
    /// of the unit that merely cites the example.
    const REUSE_ONLY_UNITS: &[&str] = &["big-o", "design-ood"];

    /// The unit whose problem pool references `slug`, if any — deterministic
    /// even when multiple units legitimately cite the same slug (see
    /// [`REUSE_ONLY_UNITS`]). Used by the FSRS review layer (Phase 6) to (a)
    /// decide whether a solved problem belongs to the course and should enter
    /// the review queue, and (b) tag a due card with its pattern for
    /// interleaving.
    pub fn unit_of_problem(&self, slug: &str) -> Option<&str> {
        let mut matches = self
            .units
            .values()
            .filter(|u| u.problems.iter().any(|p| p.slug == slug));
        let first = matches.next()?;
        if Self::REUSE_ONLY_UNITS.contains(&first.id.as_str()) {
            if let Some(owner) = matches.find(|u| !Self::REUSE_ONLY_UNITS.contains(&u.id.as_str()))
            {
                return Some(owner.id.as_str());
            }
        }
        Some(first.id.as_str())
    }

    /// Whether `slug` is a problem taught/tested anywhere in the course — the
    /// gate the review queue uses to stay Stage-1-only (the wider catalog never
    /// enters spaced review).
    pub fn is_course_problem(&self, slug: &str) -> bool {
        self.unit_of_problem(slug).is_some()
    }
}

/// Loads + validates the optional interleaved pattern-picker pool. A missing
/// file yields an empty pool (like a missing lessons dir); a present file must
/// be a well-formed quiz whose items are *all* `pattern-picker` — the pool is
/// the recognition drill, nothing else belongs in it.
fn load_pattern_pool(path: &Path) -> AppResult<Quiz> {
    if !path.is_file() {
        return Ok(Quiz { items: Vec::new() });
    }
    let pool = load_json::<Quiz>(path)?;
    pool.validate()
        .map_err(|msg| AppError::Validation(format!("{}: {msg}", path.display())))?;
    for item in &pool.items {
        if item.item_type != QuizItemType::PatternPicker {
            return Err(AppError::Validation(format!(
                "{}: pattern-pool item '{}' must be a pattern-picker",
                path.display(),
                item.id
            )));
        }
    }
    Ok(pool)
}

fn load_curriculum(path: &Path) -> AppResult<Curriculum> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| AppError::Validation(format!("{}: {e}", path.display())))?;
    let curriculum: Curriculum = serde_json::from_str(&raw).map_err(|e| {
        AppError::Validation(format!("{}: invalid curriculum JSON: {e}", path.display()))
    })?;
    curriculum
        .validate()
        .map_err(|msg| AppError::Validation(format!("{}: {msg}", path.display())))?;
    Ok(curriculum)
}

fn load_units(units_dir: &Path) -> AppResult<HashMap<String, Unit>> {
    if !units_dir.is_dir() {
        return Err(AppError::Validation(format!(
            "unit resource dir missing: {}",
            units_dir.display()
        )));
    }
    let mut entries: Vec<_> = std::fs::read_dir(units_dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());

    let mut units = HashMap::new();
    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)?;
        let unit: Unit = serde_json::from_str(&raw).map_err(|e| {
            AppError::Validation(format!("{}: invalid unit JSON: {e}", path.display()))
        })?;
        unit.validate()
            .map_err(|msg| AppError::Validation(format!("{}: {msg}", path.display())))?;
        if units.insert(unit.id.clone(), unit).is_some() {
            return Err(AppError::Validation(format!(
                "{}: duplicate unit id",
                path.display()
            )));
        }
    }
    if units.is_empty() {
        return Err(AppError::Validation(format!(
            "no unit files found under {}",
            units_dir.display()
        )));
    }
    Ok(units)
}

/// Lessons are optional content (Phase 1 ships none): a missing
/// `resources/lessons/` dir yields an empty map, not an error. Anything
/// found inside an existing dir must be well-formed.
fn load_lessons(lessons_dir: &Path) -> AppResult<HashMap<String, Lesson>> {
    if !lessons_dir.is_dir() {
        return Ok(HashMap::new());
    }
    let mut md_files = Vec::new();
    find_files(lessons_dir, "md", &mut md_files)?;

    let mut lessons = HashMap::new();
    for path in md_files {
        let lesson = load_one_lesson(&path)?;
        lesson
            .validate()
            .map_err(|msg| AppError::Validation(format!("{}: {msg}", path.display())))?;
        if lessons.insert(lesson.id.clone(), lesson).is_some() {
            return Err(AppError::Validation(format!(
                "{}: duplicate lesson id",
                path.display()
            )));
        }
    }
    Ok(lessons)
}

/// Recursively collects files with extension `ext` under `dir`, sorted for
/// deterministic load order.
fn find_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> AppResult<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            find_files(&path, ext, out)?;
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
    Ok(())
}

fn load_one_lesson(path: &Path) -> AppResult<Lesson> {
    let raw = std::fs::read_to_string(path)?;
    let (frontmatter_raw, body) = split_frontmatter(&raw).ok_or_else(|| {
        AppError::Validation(format!(
            "{}: missing YAML frontmatter (expected `---` delimiters)",
            path.display()
        ))
    })?;
    let fm: LessonFrontmatter = serde_yaml::from_str(frontmatter_raw).map_err(|e| {
        AppError::Validation(format!("{}: invalid frontmatter: {e}", path.display()))
    })?;

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let diagram = load_json::<DiagramSpec>(&dir.join(&fm.diagram))?;
    let quiz = load_json::<Quiz>(&dir.join(&fm.quiz))?;

    Ok(Lesson {
        id: fm.id,
        unit: fm.unit,
        subpattern: fm.subpattern,
        explainer_md: body.trim().to_string(),
        trigger_signals: fm.trigger_signals,
        worked_example: fm.worked_example,
        diagram,
        quiz,
        practice: fm.practice,
        recap: fm.recap,
        follow_up: fm.follow_up,
    })
}

/// Splits `---\n<frontmatter>\n---\n<body>` into `(frontmatter, body)`.
fn split_frontmatter(raw: &str) -> Option<(&str, &str)> {
    let rest = raw.strip_prefix("---")?;
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    let after = &rest[end + 4..];
    let body = after.strip_prefix('\n').unwrap_or(after);
    Some((frontmatter, body))
}

/// For each unit, its transitive prerequisite set (ancestors) over the prereq
/// DAG. Borrows unit-id strings from the curriculum. The graph is validated
/// acyclic before this runs, so the recursion terminates.
fn transitive_prereqs(curriculum: &Curriculum) -> HashMap<&str, HashSet<&str>> {
    fn collect<'a>(
        node: &'a str,
        prereqs: &'a HashMap<String, Vec<String>>,
        memo: &mut HashMap<&'a str, HashSet<&'a str>>,
    ) -> HashSet<&'a str> {
        if let Some(cached) = memo.get(node) {
            return cached.clone();
        }
        let mut acc = HashSet::new();
        if let Some(direct) = prereqs.get(node) {
            for dep in direct {
                acc.insert(dep.as_str());
                for a in collect(dep.as_str(), prereqs, memo) {
                    acc.insert(a);
                }
            }
        }
        memo.insert(node, acc.clone());
        acc
    }

    let mut memo: HashMap<&str, HashSet<&str>> = HashMap::new();
    for uid in curriculum.unit_ids() {
        collect(uid, &curriculum.prereqs, &mut memo);
    }
    memo
}

fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> AppResult<T> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| AppError::Validation(format!("{}: {e}", path.display())))?;
    serde_json::from_str(&raw)
        .map_err(|e| AppError::Validation(format!("{}: invalid JSON: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("anvil-curriculum-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn real_resources() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("resources")
    }

    fn real_packs() -> PackStore {
        PackStore::new(real_resources().join("test-packs.json.gz"))
    }

    /// Writes a minimal-but-valid curriculum tree (1 stage, 1 unit
    /// referencing `slug`, no lessons) under `dir`.
    fn write_valid_curriculum(dir: &Path, slug: &str) {
        let curriculum_dir = dir.join("curriculum");
        std::fs::create_dir_all(curriculum_dir.join("units")).unwrap();
        std::fs::write(
            curriculum_dir.join("curriculum.json"),
            r#"{
                "id": "dsa-track",
                "stages": [{ "id": "s1", "title": "Stage 1", "units": ["u1"] }],
                "prereqs": {},
                "gate_defaults": { "pass_count": 1, "require_novel": false, "timer_target_min": 10, "threshold_pct": 80 }
            }"#,
        )
        .unwrap();
        std::fs::write(
            curriculum_dir.join("units").join("u1.json"),
            format!(
                r#"{{
                "id": "u1", "stage": "s1", "title": "Unit One", "prereqs": [], "lessons": [],
                "problems": [{{ "slug": "{slug}", "role": "worked", "tier": "intro", "novel": false }}],
                "gate": {{ "pass_count": 1, "require_novel": false, "timer_target_min": 10, "threshold_pct": 80 }},
                "spiral": []
                }}"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn shipped_curriculum_loads_and_validates() {
        let store =
            CurriculumStore::load(&real_resources(), &real_packs()).expect("curriculum loads");
        // The full ladder: 8 stages, 19 units, plus the Stage-7 capstone.
        assert_eq!(store.curriculum().stages.len(), 8);
        assert!(store.get_unit("big-o").is_some());
        assert!(store.get_unit("arrays-hashing").is_some());
        assert!(store.get_unit("two-pointers").is_some());
        assert!(store.get_unit("sliding-window").is_some());
        assert!(store.get_unit("trees").is_some());
        assert!(store.get_unit("dp-1d").is_some());
        assert!(store.get_unit("design-ood").is_some());
        assert!(store.get_unit("no-such-unit").is_none());
        assert!(store.capstone().is_some(), "Stage-7 capstone ships");

        // The Phase-2 lesson loads and resolves its companion diagram/quiz.
        let lesson = store
            .get_lesson("01-hashmap-lookup")
            .expect("authored lesson loads");
        assert_eq!(lesson.worked_example, "two-sum");
        assert_eq!(lesson.diagram.for_problem, "two-sum");
        assert!(!lesson.quiz.items.is_empty());
        assert!(store
            .get_unit("arrays-hashing")
            .unwrap()
            .lessons
            .contains(&"01-hashmap-lookup".to_string()));

        // get_quiz is the lesson's quiz.
        assert_eq!(
            store.get_quiz("01-hashmap-lookup").map(|q| q.items.len()),
            Some(lesson.quiz.items.len())
        );

        // The interleaved pattern pool loads, is all pattern-picker, and every
        // item's correct_pattern names a real unit.
        let pool = store.pattern_pool();
        assert!(!pool.items.is_empty(), "shipped pattern pool present");
        for item in &pool.items {
            assert_eq!(item.item_type, QuizItemType::PatternPicker);
            let pat = item.correct_pattern.as_deref().expect("picker has pattern");
            assert!(store.get_unit(pat).is_some(), "unknown pool pattern {pat}");
        }
    }

    #[test]
    fn pattern_pool_rejects_a_non_picker_item() {
        let dir = scratch("pool-non-picker");
        write_valid_curriculum(&dir, "two-sum");
        std::fs::write(
            dir.join("curriculum").join("pattern-pool.json"),
            r#"{ "items": [{ "id": "x", "type": "complexity", "prompt_md": "p",
                 "options": ["a", "b"], "answer": "a", "explanation_md": "e" }] }"#,
        )
        .unwrap();
        let err = CurriculumStore::load(&dir, &real_packs()).unwrap_err();
        assert!(
            err.to_string().contains("must be a pattern-picker"),
            "{err}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn pattern_pool_rejects_unknown_pattern() {
        let dir = scratch("pool-unknown-pattern");
        write_valid_curriculum(&dir, "two-sum");
        std::fs::write(
            dir.join("curriculum").join("pattern-pool.json"),
            r#"{ "items": [{ "id": "x", "type": "pattern-picker", "prompt_md": "p",
                 "options": ["a", "b"], "answer": "a", "correct_pattern": "ghost-unit",
                 "explanation_md": "e" }] }"#,
        )
        .unwrap();
        let err = CurriculumStore::load(&dir, &real_packs()).unwrap_err();
        assert!(err.to_string().contains("not a known unit"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_pattern_pool_is_an_empty_pool() {
        let dir = scratch("pool-absent");
        write_valid_curriculum(&dir, "two-sum");
        let store = CurriculumStore::load(&dir, &real_packs()).expect("loads without a pool");
        assert!(store.pattern_pool().items.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_curriculum_json_fails_loudly() {
        let dir = scratch("missing-curriculum");
        let err = CurriculumStore::load(&dir, &real_packs()).unwrap_err();
        assert!(err.to_string().contains("curriculum.json"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unit_referencing_unpacked_slug_fails_loudly() {
        let dir = scratch("no-pack");
        write_valid_curriculum(&dir, "totally-not-a-real-slug-xyz");
        let err = CurriculumStore::load(&dir, &real_packs()).unwrap_err();
        assert!(err.to_string().contains("no frozen pack"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn stage_referencing_unknown_unit_fails_loudly() {
        let dir = scratch("dangling-stage");
        write_valid_curriculum(&dir, "two-sum");
        std::fs::write(
            dir.join("curriculum").join("curriculum.json"),
            r#"{
                "id": "dsa-track",
                "stages": [{ "id": "s1", "title": "Stage 1", "units": ["u1", "ghost"] }],
                "prereqs": {},
                "gate_defaults": { "pass_count": 1, "require_novel": false, "timer_target_min": 10, "threshold_pct": 80 }
            }"#,
        )
        .unwrap();
        let err = CurriculumStore::load(&dir, &real_packs()).unwrap_err();
        assert!(err.to_string().contains("unknown unit 'ghost'"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn malformed_lesson_file_fails_the_loader_loudly() {
        let dir = scratch("malformed-lesson");
        write_valid_curriculum(&dir, "two-sum");
        let lessons_dir = dir.join("lessons").join("u1");
        std::fs::create_dir_all(&lessons_dir).unwrap();
        // No `---` frontmatter delimiters at all.
        std::fs::write(
            lessons_dir.join("01-broken.md"),
            "just some prose, no frontmatter",
        )
        .unwrap();

        let err = CurriculumStore::load(&dir, &real_packs()).unwrap_err();
        assert!(err.to_string().contains("frontmatter"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn lesson_with_missing_required_part_fails_loudly() {
        let dir = scratch("missing-part");
        write_valid_curriculum(&dir, "two-sum");
        let lessons_dir = dir.join("lessons").join("u1");
        std::fs::create_dir_all(&lessons_dir).unwrap();
        std::fs::write(
            lessons_dir.join("d1.json"),
            r#"{ "id": "d1", "algorithm": "a", "for_problem": "two-sum", "mode": "view",
                 "steps": [{ "state": {}, "caption_md": "s0" }], "predict_at": [0] }"#,
        )
        .unwrap();
        std::fs::write(
            lessons_dir.join("q1.json"),
            r#"{ "items": [{ "id": "q1", "type": "complexity", "prompt_md": "p",
                 "options": ["a"], "answer": "a", "explanation_md": "e" }] }"#,
        )
        .unwrap();
        // trigger_signals is missing entirely -> empty vec -> validate() rejects it.
        std::fs::write(
            lessons_dir.join("01-lesson.md"),
            r#"---
id: 01-lesson
unit: u1
subpattern: "Thing"
worked_example: two-sum
diagram: d1.json
quiz: q1.json
practice: [two-sum]
---
Prose.
"#,
        )
        .unwrap();

        let err = CurriculumStore::load(&dir, &real_packs()).unwrap_err();
        assert!(err.to_string().contains("trigger signal"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn well_formed_lesson_loads_successfully() {
        let dir = scratch("valid-lesson");
        write_valid_curriculum(&dir, "two-sum");
        let lessons_dir = dir.join("lessons").join("u1");
        std::fs::create_dir_all(&lessons_dir).unwrap();
        std::fs::write(
            lessons_dir.join("d1.json"),
            r#"{ "id": "d1", "algorithm": "a", "for_problem": "two-sum", "mode": "view",
                 "steps": [{ "state": {}, "caption_md": "s0" }], "predict_at": [0] }"#,
        )
        .unwrap();
        std::fs::write(
            lessons_dir.join("q1.json"),
            r#"{ "items": [{ "id": "q1", "type": "complexity", "prompt_md": "p",
                 "options": ["a"], "answer": "a", "explanation_md": "e" }] }"#,
        )
        .unwrap();
        std::fs::write(
            lessons_dir.join("01-lesson.md"),
            r#"---
id: 01-lesson
unit: u1
subpattern: "Thing"
trigger_signals: ["seen it before"]
worked_example: two-sum
diagram: d1.json
quiz: q1.json
practice: [two-sum]
---
The explainer prose goes here.
"#,
        )
        .unwrap();
        // The unit must list the lesson id for the cross-check to pass.
        std::fs::write(
            dir.join("curriculum").join("units").join("u1.json"),
            r#"{
                "id": "u1", "stage": "s1", "title": "Unit One", "prereqs": [], "lessons": ["01-lesson"],
                "problems": [{ "slug": "two-sum", "role": "worked", "tier": "intro", "novel": false }],
                "gate": { "pass_count": 1, "require_novel": false, "timer_target_min": 10, "threshold_pct": 80 },
                "spiral": []
            }"#,
        )
        .unwrap();

        let store = CurriculumStore::load(&dir, &real_packs()).expect("loads cleanly");
        let lesson = store.get_lesson("01-lesson").expect("lesson present");
        assert_eq!(lesson.explainer_md, "The explainer prose goes here.");
        assert_eq!(lesson.worked_example, "two-sum");
        std::fs::remove_dir_all(&dir).ok();
    }
}
