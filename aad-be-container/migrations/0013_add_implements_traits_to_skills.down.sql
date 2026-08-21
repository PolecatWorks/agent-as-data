-- Migration 0013: Rollback add implements_traits to skills
DROP INDEX IF EXISTS idx_skills_traits;
ALTER TABLE skills DROP COLUMN IF EXISTS implements_traits;
