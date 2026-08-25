ALTER TABLE agents ADD COLUMN uses_traits TEXT[] DEFAULT '{}';
ALTER TABLE skills ADD COLUMN uses_traits TEXT[] DEFAULT '{}';
