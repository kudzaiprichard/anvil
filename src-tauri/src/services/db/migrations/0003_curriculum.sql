-- Course per-user state. Curriculum/unit/
-- lesson *content* is bundled resource data (validated by
-- services::curriculum); these tables hold only what the user *does* with
-- it, same split as `problem_state` vs. the catalog. Columns are stubbed
-- ahead of the phases that write them: Phase 2 writes `lesson_progress`,
-- Phase 3 writes `unit_mastery`, Phase 6 writes `review_schedule`.

CREATE TABLE lesson_progress (
  lesson_id TEXT PRIMARY KEY,
  unit_id TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'not-started' CHECK (status IN ('not-started','in-progress','complete')),
  started_at TEXT,
  completed_at TEXT
);

CREATE INDEX idx_lesson_progress_unit_id ON lesson_progress(unit_id);

CREATE TABLE unit_mastery (
  unit_id TEXT PRIMARY KEY,
  status TEXT NOT NULL DEFAULT 'locked' CHECK (status IN ('locked','unlocked','mastered')),
  gate_attempts INTEGER NOT NULL DEFAULT 0,
  gate_passed_count INTEGER NOT NULL DEFAULT 0,
  gate_passed_novel INTEGER NOT NULL DEFAULT 0,
  mastered_at TEXT
);

-- FSRS scheduler state (fsrs-rs, Phase 6), one row per problem that has
-- entered the review queue (gate-passed or lesson-solved).
CREATE TABLE review_schedule (
  problem_id TEXT PRIMARY KEY,
  state TEXT NOT NULL DEFAULT 'new' CHECK (state IN ('new','learning','review','relearning')),
  stability REAL,
  difficulty REAL,
  due_at TEXT,
  last_reviewed_at TEXT,
  lapses INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_review_schedule_due_at ON review_schedule(due_at);
