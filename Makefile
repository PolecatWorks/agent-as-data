.PHONY: help dev db-up db-down stop-other-db test build-docker garden-up

DATABASE_URL ?= postgres://postgres:postgres@localhost:5432/agent_as_data

help:
	@echo "Agent-As-Data Make Targets:"
	@echo "  dev           - Run Rust backend dev server"
	@echo "  db-up         - Start Postgres via Docker Compose"
	@echo "  db-down       - Stop Postgres"
	@echo "  stop-other-db - Stop conflicting Postgres container (sward-postgres)"
	@echo "  test          - Run cargo test"
	@echo "  build-docker  - Build backend docker image"
	@echo "  garden-up     - Deploy dev environment via Garden"

stop-other-db:
	docker stop sward-postgres 2>/dev/null || true

db-up:
	docker compose -f docker-compose/docker-compose.yaml up -d

db-down:
	docker compose -f docker-compose/docker-compose.yaml down

dev:
	-@lsof -t -i :8085 | xargs kill -9 2>/dev/null || true
	cd aad-be-container && \
	AAD_BE__DATABASE__URL__URL="postgresql://localhost:5432/agent_as_data" \
	AAD_BE__DATABASE__URL__USERNAME="postgres" \
	AAD_BE__DATABASE__URL__PASSWORD="postgres" \
	cargo run -- serve

test:
	cd aad-be-container && cargo test

build-docker:
	docker build -t agent-as-data-be:latest aad-be-container

garden-up:
	garden deploy --env local
