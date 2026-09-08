.PHONY: help all dev aad-be-dev aad-be-watch aad-be-migrate aad-be-test aad-be-docker aad-be-docker-run \
        aad-mcp-dev aad-mcp-watch aad-mcp-test aad-mcp-docker aad-mcp-docker-run \
        aad-fe-dev aad-fe-test aad-fe-docker aad-fe-docker-run \
        db-up db-down compose-db-up compose-db-down compose-db-clean stop-other-db \
        test build-be build-mcp build-fe build-docker garden-up garden-test garden-down robot-test

DATABASE_URL ?= postgres://postgres:mysecretpassword@localhost:5432/aaddb

RUST_APPS := aad-be aad-mcp
NODE_APPS := aad-fe

aad-be_PORT ?= 8080
aad-be_HEALTH_PORT ?= 8079
aad-mcp_PORT ?= 8082
aad-mcp_HEALTH_PORT ?= 8078
aad-fe_PORT ?= 4200
ROBOT_REPORT_DIR ?= integration-tests/reports
GHCR_USER ?= bengreen
GHCR_TOKEN ?= $(shell gh auth token 2>/dev/null)

all: build-be build-mcp build-fe

help:
	@echo "Agent-As-Data Make Targets:"
	@echo "  help               - Display this help message"
	@echo "  dev                - Run Rust backend dev server (alias for aad-be-dev)"
	@echo "  aad-be-dev         - Run Rust backend dev server with auto-port cleanup"
	@echo "  aad-be-watch       - Run Rust backend with cargo watch auto-recompilation"
	@echo "  aad-be-migrate     - Run database migrations against PostgreSQL"
	@echo "  aad-be-test        - Run backend unit tests via cargo test"
	@echo "  aad-be-docker      - Build Rust backend Docker image"
	@echo "  aad-be-docker-run  - Build and run backend container locally"
	@echo "  aad-mcp-dev        - Run Rust MCP dev server with auto-port cleanup"
	@echo "  aad-mcp-watch      - Run Rust MCP server with cargo watch auto-recompilation"
	@echo "  aad-mcp-test       - Run MCP server unit tests via cargo test"
	@echo "  aad-mcp-docker     - Build Rust MCP Docker image"
	@echo "  aad-mcp-docker-run - Build and run MCP container locally"
	@echo "  ensure-pgvector    - Ensure pgvector extension is installed in PostgreSQL"
	@echo "  aad-fe-dev         - Run Angular frontend dev server"
	@echo "  aad-fe-test        - Run Angular frontend unit tests via Karma/ChromeHeadless"
	@echo "  aad-fe-docker      - Build Angular frontend Docker image"
	@echo "  aad-fe-docker-run  - Build and run frontend container locally"
	@echo "  db-up              - Alias for compose-db-up (Start Postgres via Docker Compose)"
	@echo "  db-down            - Alias for compose-db-down (Stop Postgres)"
	@echo "  compose-db-up      - Start Postgres container via docker-compose/postgres.yaml"
	@echo "  compose-db-down    - Stop Postgres container"
	@echo "  compose-db-clean   - Stop Postgres container and remove volumes"
	@echo "  stop-other-db      - Stop conflicting Postgres container (sward-postgres)"
	@echo "  test               - Run backend unit tests via cargo test (alias for aad-be-test)"
	@echo "  build-be           - Build backend Docker image (agent-as-data-be:latest)"
	@echo "  build-mcp          - Build MCP server Docker image (agent-as-data-mcp:latest)"
	@echo "  build-fe           - Build frontend Docker image (agent-as-data-fe:latest)"
	@echo "  build-docker       - Alias for build-be"
	@echo "  garden-up          - Deploy dev environment via Garden"
	@echo "  garden-test        - Run integration tests inside Garden local cluster"
	@echo "  garden-down        - Tear down Garden local dev environment"
	@echo "  robot-test         - Execute Robot Framework integration test runner"
	@echo "  seed-data          - Seed the database with exemplar data (Traits, Tools, Skills, Agents)"

stop-other-db:
	docker stop sward-postgres 2>/dev/null || true

compose-db-up:
	docker compose -f docker-compose/postgres.yaml up -d --wait
	@docker exec -i aad-postgres psql -U postgres -d aaddb -c "CREATE EXTENSION IF NOT EXISTS vector;"

compose-db-down:
	docker compose -f docker-compose/postgres.yaml down

ensure-pgvector:
	@docker exec -i aad-postgres psql -U postgres -d aaddb -c "CREATE EXTENSION IF NOT EXISTS vector;"

compose-db-clean:
	docker compose -f docker-compose/postgres.yaml down -v

db-up: compose-db-up

db-down: compose-db-down

dev: aad-be-dev

aad-be-dev:
	-@lsof -t -i :$(aad-be_PORT) | xargs kill -9 2>/dev/null || true
	-@lsof -t -i :$(aad-be_HEALTH_PORT) | xargs kill -9 2>/dev/null || true
	cd aad-be-container && \
	DATABASE_URL="$(DATABASE_URL)" \
	AAD_BE__WEBSERVICE__ADDRESS="0.0.0.0:$(aad-be_PORT)" \
	AAD_BE__HAMS__PORT="$(aad-be_HEALTH_PORT)" \
	cargo run -- serve

aad-be-watch:
	-@lsof -t -i :$(aad-be_PORT) | xargs kill -9 2>/dev/null || true
	-@lsof -t -i :$(aad-be_HEALTH_PORT) | xargs kill -9 2>/dev/null || true
	cd aad-be-container && \
	DATABASE_URL="$(DATABASE_URL)" \
	AAD_BE__WEBSERVICE__ADDRESS="0.0.0.0:$(aad-be_PORT)" \
	AAD_BE__HAMS__PORT="$(aad-be_HEALTH_PORT)" \
	AAD_BE__DEBUGGING__LOG_LEVEL="debug" \
	RUST_LOG="debug" \
	cargo watch -x 'run -- serve'


aad-be-migrate:
	cd aad-be-container && \
	DATABASE_URL="$(DATABASE_URL)" \
	cargo run -- migrate

aad-be-test:
	cd aad-be-container && cargo test

aad-be-docker:
	docker build -t agent-as-data-be:latest aad-be-container

aad-be-docker-run: aad-be-docker
	docker run -it --rm --name agent-as-data-be \
		-p $(aad-be_PORT):8080 \
		-p $(aad-be_HEALTH_PORT):8079 \
		agent-as-data-be:latest

aad-mcp-dev:
	-@lsof -t -i :$(aad-mcp_PORT) | xargs kill -9 2>/dev/null || true
	-@lsof -t -i :$(aad-mcp_HEALTH_PORT) | xargs kill -9 2>/dev/null || true
	cd aad-mcp-container && \
	AAD_MCP__WEBSERVICE__ADDRESS="0.0.0.0:$(aad-mcp_PORT)" \
	AAD_MCP__HAMS__ADDRESS="0.0.0.0:$(aad-mcp_HEALTH_PORT)" \
	cargo run -- serve

aad-mcp-watch:
	-@lsof -t -i :$(aad-mcp_PORT) | xargs kill -9 2>/dev/null || true
	-@lsof -t -i :$(aad-mcp_HEALTH_PORT) | xargs kill -9 2>/dev/null || true
	cd aad-mcp-container && \
	AAD_MCP__WEBSERVICE__ADDRESS="0.0.0.0:$(aad-mcp_PORT)" \
	AAD_MCP__HAMS__ADDRESS="0.0.0.0:$(aad-mcp_HEALTH_PORT)" \
	AAD_MCP__DEBUGGING__LOG_LEVEL="debug" \
	RUST_LOG="debug" \
	cargo watch -x 'run -- serve'

aad-mcp-test:
	cd aad-mcp-container && cargo test

aad-mcp-docker:
	docker build -t agent-as-data-mcp:latest aad-mcp-container

aad-mcp-docker-run: aad-mcp-docker
	docker run -it --rm --name agent-as-data-mcp \
		-p $(aad-mcp_PORT):8080 \
		-p $(aad-mcp_HEALTH_PORT):8079 \
		agent-as-data-mcp:latest

aad-fe-dev:
	-@lsof -t -i :$(aad-fe_PORT) | xargs kill -9 2>/dev/null || true
	cd aad-fe-container && npm start

aad-fe-test:
	cd aad-fe-container && npm test -- --watch=false --browsers=ChromeHeadless

aad-fe-docker:
	docker build -t agent-as-data-fe:latest aad-fe-container

aad-fe-docker-run: aad-fe-docker
	docker run -it --rm --name agent-as-data-fe \
		-p $(aad-fe_PORT):80 \
		agent-as-data-fe:latest

test: aad-be-test

build-be: aad-be-docker

build-mcp: aad-mcp-docker

build-fe: aad-fe-docker

build-docker: build-be build-mcp build-fe

garden-up:
	@echo "Logging Helm into GHCR and running Garden deploy..."
	@echo "$${GHCR_TOKEN:-$(GHCR_TOKEN)}" | helm registry login ghcr.io -u "$${GHCR_USER:-$(GHCR_USER)}" --password-stdin 2>/dev/null || true
	GHCR_TOKEN="$${GHCR_TOKEN:-$(GHCR_TOKEN)}" GHCR_USER="$${GHCR_USER:-$(GHCR_USER)}" garden deploy --env local

garden-test: garden-up
	@echo "Running Garden tests..."
	GHCR_TOKEN="$${GHCR_TOKEN:-$(GHCR_TOKEN)}" GHCR_USER="$${GHCR_USER:-$(GHCR_USER)}" garden test --env local
	@echo "Copying test reports to $(ROBOT_REPORT_DIR)..."
	@mkdir -p $(ROBOT_REPORT_DIR)
	@NS="agent-as-data-$${USER:-local}"; \
	kubectl cp $$NS/robot-test-runner:/tmp/reports $(ROBOT_REPORT_DIR) 2>/dev/null || true
	@if [ -f "$(ROBOT_REPORT_DIR)/log.html" ]; then \
		echo "Opening test report log.html..."; \
		open $(ROBOT_REPORT_DIR)/log.html || true; \
	fi

garden-down:
	@echo "Tearing down Garden environment..."
	garden cleanup env --env local

robot-test:
	./integration-tests/run-tests-local.sh

seed-data:
	./integration-tests/run-tests-local.sh tests/test_seed_exemplar_data.robot
