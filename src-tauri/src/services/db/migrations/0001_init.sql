-- Initial schema (BACKEND_PLAN §5.2). Timestamps are local-time ISO-8601
-- strings; streaks/heatmaps bucket by the leading yyyy-mm-dd.

CREATE TABLE user_problems (
  id TEXT PRIMARY KEY,
  number INTEGER NOT NULL,
  json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE attempts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  problem_id TEXT NOT NULL,
  language TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('run','submit')),
  status TEXT NOT NULL CHECK (status IN ('pass','fail','error','timeout')),
  runtime_ms INTEGER,
  attempted_at TEXT NOT NULL
);

CREATE INDEX idx_attempts_problem_id ON attempts(problem_id);
CREATE INDEX idx_attempts_attempted_at ON attempts(attempted_at);

CREATE TABLE problem_state (
  problem_id TEXT PRIMARY KEY,
  status TEXT NOT NULL DEFAULT 'todo',
  mastered INTEGER NOT NULL DEFAULT 0,
  bookmarked INTEGER NOT NULL DEFAULT 0,
  last_attempted_at TEXT,
  last_code TEXT,
  last_language TEXT
);

CREATE TABLE drafts (
  id TEXT PRIMARY KEY,
  json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
