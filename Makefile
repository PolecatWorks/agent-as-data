.PHONY: help all dev db-up db-down compose-db-up compose-db-down compose-db-clean stop-other-db test build-be build-fe build-docker garden-up robot-test

DATABASE_URL ?= postgres://postgres:mysecretpassword@localhost:5432/aaddb

all: build-be build-fe

help:
	@echo "Agent-As-Data Make Targets:"
	@echo "  help             - Display this help message"
	@echo "  dev              - Run Rust backend dev server"
	@echo "  db-up            - Alias for compose-db-up (Start Postgres via Docker Compose)"
	@echo "  db-down          - Alias for compose-db-down (Stop Postgres)"
	@echo "  compose-db-up    - Start Postgres container via docker-compose/postgres.yaml"
	@echo "  compose-db-down  - Stop Postgres container"
	@echo "  compose-db-clean - Stop Postgres container and remove volumes"
	@echo "  stop-other-db    - Stop conflicting Postgres container (sward-postgres)"
	@echo "  test             - Run backend unit tests via cargo test"
	@echo "  build-be         - Build backend Docker image (agent-as-data-be:latest)"
	@echo "  build-fe         - Build frontend Docker image (agent-as-data-fe:latest)"
	@echo "  build-docker     - Alias for build-be"
	@echo "  garden-up        - Deploy dev environment via Garden"
	@echo "  robot-test       - Execute Robot Framework integration test runner"

stop-other-db:
	docker stop sward-postgres 2>/dev/null || true

compose-db-up:
	docker compose -f docker-compose/postgres.yaml up -d

compose-db-down:
	docker compose -f docker-compose/postgres.yaml down

compose-db-clean:
	docker compose -f docker-compose/postgres.yaml down -v

db-up: compose-db-up

db-down: compose-db-down

dev:
	-@lsof -t -i :8080 | xargs kill -9 2>/dev/null || true
	-@lsof -t -i :8079 | xargs kill -9 2>/dev/null || true
	cd aad-be-container && \
	AAD_BE__DATABASE__URL="postgres://postgres:mysecretpassword@localhost:5432/aaddb" \
	cargo run -- serve

test:
	cd aad-be-container && cargo test

build-be:
	docker build -t agent-as-data-be:latest aad-be-container

build-fe:
	docker build -t agent-as-data-fe:latest aad-fe-container

build-docker: build-be

garden-up:
	garden deploy --env local

robot-test:
	./integration-tests/run-tests-local.sh
