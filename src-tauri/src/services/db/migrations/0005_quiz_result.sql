-- Formative quiz outcomes (Phase 4).
-- Quizzes are NEVER a gate: this table only *feeds the review signal* — which
-- items a learner got right/wrong, so later phases (FSRS review scheduling,
-- readiness signal) can lean on real recognition data. Quiz *content* is
-- bundled resource data (validated by services::curriculum); this holds only
-- what the user did with it, the same content/state split as lesson_progress.
--
-- One row per answered item per submission (append-only, not upserted): a
-- learner may re-take a formative check, and the history is the signal.
CREATE TABLE quiz_result (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  -- Where the item came from: a lesson id, or 'pattern-pool' for the
  -- interleaved cross-unit pattern-picker pool.
  source TEXT NOT NULL,
  item_id TEXT NOT NULL,
  item_type TEXT NOT NULL CHECK (item_type IN ('concept-check','pattern-picker','complexity')),
  correct INTEGER NOT NULL CHECK (correct IN (0, 1)),
  answered_at TEXT NOT NULL
);

CREATE INDEX idx_quiz_result_source ON quiz_result(source);
