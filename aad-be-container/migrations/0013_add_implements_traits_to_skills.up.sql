-- Migration 0013: Add implements_traits to skills
ALTER TABLE skills ADD COLUMN implements_traits TEXT[] NOT NULL DEFAULT '{}';
CREATE INDEX IF NOT EXISTS idx_skills_traits ON skills USING GIN(implements_traits);
