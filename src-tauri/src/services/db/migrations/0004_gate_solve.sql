-- Phase 3: mastery-gate solves (COURSE_BLUEPRINT.md §6, LESSON_COURSE_DESIGN.md
-- §3.6). One row per gate problem a learner has cleared *hint-free and no-peek*
-- — the only solves that count toward mastery. The PRIMARY KEY makes a re-solve
-- idempotent (it never double-counts), and `novel` mirrors the unit manifest's
-- tag so the engine can enforce "pass = N incl. >=1 novel". `unit_mastery`
-- (0003) still holds the derived per-unit status/attempt counters; this table
-- is the fine-grained evidence the engine tallies. Same content/state split as
-- everything else in `services::db`: curriculum content is bundled resource
-- data, this is only what the user *did*.

CREATE TABLE gate_solve (
  unit_id TEXT NOT NULL,
  problem_id TEXT NOT NULL,
  novel INTEGER NOT NULL DEFAULT 0,
  solved_at TEXT NOT NULL,
  PRIMARY KEY (unit_id, problem_id)
);

CREATE INDEX idx_gate_solve_unit ON gate_solve(unit_id);
