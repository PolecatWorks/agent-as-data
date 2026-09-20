ALTER TABLE knowledge_nodes
ADD COLUMN description TEXT,
ADD COLUMN tags TEXT[] NOT NULL DEFAULT '{}';
