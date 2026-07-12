-- Closing-the-48: record the experience tier a problem was judged under at
-- the moment of each attempt ("full" = hidden pack, "basic" = statement
-- examples only, "run-only" = no verdict). A Basic-mode pass is a much weaker
-- signal than a Full-tier pass; recording it per attempt keeps history honest
-- and lets later UI distinguish or re-qualify passes once a pack lands.
-- Pre-existing rows default to 'full': until now the overwhelming majority of
-- recorded attempts were pack-backed or built-in problems with hidden cases.
ALTER TABLE attempts ADD COLUMN tier TEXT NOT NULL DEFAULT 'full';
