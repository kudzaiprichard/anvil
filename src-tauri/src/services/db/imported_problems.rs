//! Imported-problem persistence (task 0005). Mirrors `user_problems` (full
//! `Problem` stored as JSON) but adds the import metadata the importer and
//! Library need — tier, preset tags, and the `scraped_at` recency key that
//! makes re-import idempotent. Keyed by slug; the slug is also the stored
//! `Problem.id`, so attempt history survives a re-import unchanged.

use std::collections::HashMap;

use rusqlite::params;

use super::Db;
use crate::domain::problem::Problem;
use crate::error::{AppError, AppResult};

/// What we already hold for a slug — enough to decide insert vs. update vs.
/// leave-alone on re-import without deserializing the whole record.
#[derive(Debug, Clone)]
pub struct ExistingMeta {
    pub number: u32,
    /// The scrape timestamp the stored copy came from (newest wins).
    pub scraped_at: Option<String>,
}

/// Slug → metadata for every imported problem, for the upsert decision.
pub fn existing(db: &Db) -> AppResult<HashMap<String, ExistingMeta>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare("SELECT slug, number, scraped_at FROM imported_problems")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            ExistingMeta {
                number: row.get::<_, i64>(1)? as u32,
                scraped_at: row.get::<_, Option<String>>(2)?,
            },
        ))
    })?;
    rows.collect::<Result<HashMap<_, _>, _>>()
        .map_err(Into::into)
}

/// Inserts or replaces one imported problem. `imported_at` is preserved on
/// update (the first-seen time); `updated_at` is the current write.
#[allow(clippy::too_many_arguments)]
pub fn upsert(
    db: &Db,
    slug: &str,
    qid: &str,
    number: u32,
    tier: &str,
    presets: &[String],
    scraped_at: &str,
    problem: &Problem,
    now: &str,
) -> AppResult<()> {
    let json = serde_json::to_string(problem)
        .map_err(|e| AppError::Storage(format!("failed to encode imported problem: {e}")))?;
    let presets_json = serde_json::to_string(presets)
        .map_err(|e| AppError::Storage(format!("failed to encode preset tags: {e}")))?;
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO imported_problems
           (slug, qid, number, tier, presets, scraped_at, json, imported_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
         ON CONFLICT(slug) DO UPDATE SET
           qid = excluded.qid,
           number = excluded.number,
           tier = excluded.tier,
           presets = excluded.presets,
           scraped_at = excluded.scraped_at,
           json = excluded.json,
           updated_at = excluded.updated_at",
        params![
            slug,
            qid,
            number as i64,
            tier,
            presets_json,
            scraped_at,
            json,
            now
        ],
    )?;
    Ok(())
}

/// Every imported problem, ordered by number (merged into the library by
/// `ProblemStore`).
pub fn list(db: &Db) -> AppResult<Vec<Problem>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare("SELECT json FROM imported_problems ORDER BY number")?;
    let rows: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    rows.iter()
        .map(|json| {
            serde_json::from_str(json)
                .map_err(|e| AppError::Storage(format!("corrupt imported problem record: {e}")))
        })
        .collect()
}

/// Slug → preset tags, for the Library preset filter.
pub fn preset_tags(db: &Db) -> AppResult<HashMap<String, Vec<String>>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare("SELECT slug, presets FROM imported_problems")?;
    let rows = stmt.query_map([], |row| {
        let slug: String = row.get(0)?;
        let presets: String = row.get(1)?;
        Ok((slug, presets))
    })?;
    let mut out = HashMap::new();
    for row in rows {
        let (slug, presets) = row?;
        let tags: Vec<String> = serde_json::from_str(&presets).unwrap_or_default();
        out.insert(slug, tags);
    }
    Ok(out)
}

/// `(count, most-recent import timestamp)` for the manage controls.
pub fn stats(db: &Db) -> AppResult<(u32, Option<String>)> {
    let conn = db.lock()?;
    conn.query_row(
        "SELECT COUNT(*), MAX(updated_at) FROM imported_problems",
        [],
        |row| {
            Ok((
                row.get::<_, i64>(0)? as u32,
                row.get::<_, Option<String>>(1)?,
            ))
        },
    )
    .map_err(Into::into)
}

/// Removes every imported problem AND its attempt history / saved state —
/// "Clear all imported" is destructive by design (the UI confirms first).
/// Built-in and user problems are untouched. Returns how many were removed.
pub fn clear_all(db: &Db) -> AppResult<u32> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    let slugs: Vec<String> = {
        let mut stmt = tx.prepare("SELECT slug FROM imported_problems")?;
        let rows: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
        rows
    };
    for slug in &slugs {
        // The slug is the problem id used by attempts/state.
        tx.execute("DELETE FROM attempts WHERE problem_id = ?1", params![slug])?;
        tx.execute(
            "DELETE FROM problem_state WHERE problem_id = ?1",
            params![slug],
        )?;
    }
    tx.execute("DELETE FROM imported_problems", [])?;
    tx.commit()?;
    Ok(slugs.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::problem::{Difficulty, FunctionSignature, Pattern, ProblemSource, TestCase};

    fn problem(slug: &str) -> Problem {
        Problem {
            id: slug.into(),
            number: 5000,
            title: "Imported".into(),
            pattern: Pattern("Arrays & Hashing".into()),
            difficulty: Difficulty::Easy,
            source: ProblemSource::Imported,
            description_md: "d".into(),
            body_html: Some("<p>d</p>".into()),
            constraints: vec![],
            examples: vec![],
            function_signature: FunctionSignature {
                python: "class Solution:\n    def f(self, x): ...".into(),
                javascript: "var f = function(x) {};".into(),
                extra: Default::default(),
            },
            test_cases: vec![TestCase {
                input: vec![1.into()],
                expected: 1.into(),
                hidden: false,
            }],
            checker: crate::domain::problem::Checker::Exact,
            judge: None,
            entry_point: None,
            hints: vec![],
            reference_solution: None,
            explanation_md: None,
            follow_up: None,
            license: "user-import".into(),
            author: "imported".into(),
        }
    }

    fn temp_db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path()).unwrap();
        (dir, db)
    }

    #[test]
    fn upsert_then_list_round_trips() {
        let (_dir, db) = temp_db();
        upsert(
            &db,
            "two-sum",
            "1",
            5000,
            "full",
            &["blind75".into()],
            "2026-06-12T00:00:00Z",
            &problem("two-sum"),
            "2026-06-22T00:00:00",
        )
        .unwrap();
        let listed = list(&db).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, "two-sum");
        let (count, last) = stats(&db).unwrap();
        assert_eq!(count, 1);
        assert!(last.is_some());
        let tags = preset_tags(&db).unwrap();
        assert_eq!(tags["two-sum"], vec!["blind75".to_string()]);
    }

    #[test]
    fn upsert_is_idempotent_by_slug() {
        let (_dir, db) = temp_db();
        for _ in 0..3 {
            upsert(
                &db,
                "two-sum",
                "1",
                5000,
                "full",
                &[],
                "2026-06-12T00:00:00Z",
                &problem("two-sum"),
                "now",
            )
            .unwrap();
        }
        assert_eq!(list(&db).unwrap().len(), 1);
        let meta = existing(&db).unwrap();
        assert_eq!(meta["two-sum"].number, 5000);
        assert_eq!(
            meta["two-sum"].scraped_at.as_deref(),
            Some("2026-06-12T00:00:00Z")
        );
    }

    #[test]
    fn clear_all_removes_problems_and_their_history() {
        let (_dir, db) = temp_db();
        upsert(
            &db,
            "two-sum",
            "1",
            5000,
            "full",
            &[],
            "s",
            &problem("two-sum"),
            "now",
        )
        .unwrap();
        // seed an attempt + state row keyed by the slug
        super::super::attempts::record_attempt(
            &db,
            &super::super::attempts::AttemptRecord {
                problem_id: "two-sum",
                language: "python",
                kind: "submit",
                status: "pass",
                tier: "full",
                runtime_ms: Some(10),
                code: "print(1)",
                attempted_at: "2026-06-22T00:00:00",
            },
        )
        .unwrap();
        let removed = clear_all(&db).unwrap();
        assert_eq!(removed, 1);
        assert!(list(&db).unwrap().is_empty());
        // history gone too
        let conn = db.lock().unwrap();
        let attempts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM attempts WHERE problem_id = 'two-sum'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(attempts, 0);
    }
}
