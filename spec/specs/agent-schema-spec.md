# Agent Schema Specification

## Schema Design (PostgreSQL / JSON Schema)

### Table: `agents`

```sql
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    input_guardrails JSONB NOT NULL DEFAULT '{}'::jsonb,
    output_guardrails JSONB NOT NULL DEFAULT '{}'::jsonb,
    agent_definition JSONB NOT NULL DEFAULT '{}'::jsonb,
    model JSONB NOT NULL DEFAULT '{}'::jsonb,
    tools JSONB NOT NULL DEFAULT '[]'::jsonb,
    available_skills JSONB NOT NULL DEFAULT '[]'::jsonb,
    available_agents JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agents_name ON agents(name);
CREATE INDEX IF NOT EXISTS idx_agents_tags ON agents USING GIN(tags);
```

### Struct JSON Example

```json
{
  "id": "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11",
  "name": "code-reviewer",
  "description": "Automated code reviewer for Rust and TypeScript projects",
  "tags": ["coding", "review", "rust"],
  "input_guardrails": {
    "max_token_length": 8192,
    "forbidden_keywords": ["SECRET_KEY", "PRIVATE_KEY"],
    "allowed_file_types": ["rs", "ts", "md"]
  },
  "output_guardrails": {
    "require_markdown_format": true,
    "max_suggestions": 10
  },
  "agent_definition": {
    "system_prompt": "You are a senior staff engineer performing code reviews.",
    "temperature": 0.2
  },
  "model": {
    "provider": "anthropic",
    "model_name": "claude-3-5-sonnet-20241022",
    "max_tokens": 4096
  },
  "tools": [
    {
      "name": "cargo_check",
      "description": "Runs cargo check on workspace"
    }
  ],
  "available_skills": [
    {
      "name": "security-audit",
      "version": "1.0.0"
    }
  ],
  "available_agents": [
    {
      "name": "linter-agent",
      "id": "b1eebc99-9c0b-4ef8-bb6d-6bb9bd380a22"
    }
  ]
}
```
