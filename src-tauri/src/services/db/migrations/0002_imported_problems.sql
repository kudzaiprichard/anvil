-- Imported LeetCode problems (task 0005). One row per
-- slug = the user's statement merged with our pre-built test pack. Keyed by
-- slug so re-imports upsert in place and attempt history (in `attempts` /
-- `problem_state`, keyed by the slug used as the problem id) survives.
--
-- `tier`, `presets`, and `scraped_at` are import metadata that does NOT live
-- in the serialized Problem: `tier` drives the basic/run-only UI hint,
-- `presets` drives the Library preset filter, `scraped_at` is the upsert
-- recency key (newest scrape wins). `json` is the full Problem record, the
-- same storage shape as `user_problems`.
CREATE TABLE imported_problems (
  slug TEXT PRIMARY KEY,
  qid TEXT,
  number INTEGER NOT NULL,
  tier TEXT NOT NULL,
  presets TEXT NOT NULL DEFAULT '[]',
  scraped_at TEXT,
  json TEXT NOT NULL,
  imported_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
