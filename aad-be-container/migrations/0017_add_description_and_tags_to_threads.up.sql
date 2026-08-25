ALTER TABLE threads
ADD COLUMN description TEXT,
ADD COLUMN tags JSONB DEFAULT '[]'::jsonb;
