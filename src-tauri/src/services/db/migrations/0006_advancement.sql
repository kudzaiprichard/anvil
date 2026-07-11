-- Phase 7 advanced progression.
--
-- Diagnostic placement: a unit can be mastered either by passing its gate or by
-- being *placed out* of it (the learner demonstrated recognition in the
-- diagnostic). `placed` distinguishes the two so the readiness signal and UI can
-- stay honest about which mastery was earned at the gate vs. tested out of.
ALTER TABLE unit_mastery ADD COLUMN placed INTEGER NOT NULL DEFAULT 0;

-- Stage-7 mixed capstone: one row per capstone problem cleared hint-free. Kept
-- separate from `gate_solve` because the capstone is not a unit — it spans all
-- of them, unlabeled.
CREATE TABLE capstone_solve (
  problem_id TEXT PRIMARY KEY,
  solved_at TEXT NOT NULL
);
