# Agent-As-Data (AAD)

Agent-As-Data is a declarative platform and specification for representing AI agents as structured, queryable data in PostgreSQL rather than imperative code.

---

## Directory & Repository Structure

- `aad-be-container/`: Rust backend microservice (`Axum`, `SQLx`, `pgvector`, `Tokio`, `HaMS`).
- `aad-fe-container/`: Angular 18+ Developer UI Studio web dashboard.
- `spec/`: Specifications, Product Requirement Documents (PRDs), research notes, and DDL schema references.
  - `spec/prds/`: Master product requirements and domain PRDs.
  - `spec/specs/`: Test-Driven Development (TDD) build specs (`01` through `08`).
  - `spec/research/`: Research knowledge base and technical architecture explorations.
- `integration-tests/`: Robot Framework end-to-end integration test runner and journey suites.
- `docker-compose/`: Local PostgreSQL container stack configuration (`postgres.yaml`).
- `charts/agent-as-data/`: Helm deployment chart for Kubernetes environments.
- `fluxcd-dev/`: FluxCD Kustomization & HelmRelease dev manifests.
- `garden.yml`: Garden configuration for local Kubernetes development & testing.
- `.github/workflows/`: CI workflows for Rust compilation, unit testing, and Helm linting.

---

## Local Development & Execution Guide

### Prerequisites
- **Rust Toolchain**: `1.85+` (with `cargo`)
- **Docker & Docker Compose**: For local PostgreSQL database container with `pgvector`
- **Python 3 & Robot Framework**: `pip install robotframework requests` (for running integration tests)
- **Node.js 20+**: `npm` (for running Angular frontend dev server)

---

### Step-by-Step Local Execution Workflow

#### 1. Start Local Database Stack (PostgreSQL + `pgvector`)
Start the local PostgreSQL container on port `5432`:
```bash
make db-up
```
> **Note**: To stop or clean up database volumes, use `make db-down` or `make compose-db-clean`.

#### 2. Run Database Migrations
Apply automatic schema migrations (`.up.sql` scripts) against the database:
```bash
make aad-be-migrate
```

#### 3. Run the Rust Backend Microservice
Launch the backend REST API server on port `8080` (with HaMS health sidecar on port `8079`):
```bash
make dev
```
Alternatively, run with hot-reloading watch mode (`cargo watch`):
```bash
make aad-be-watch
```

#### 4. Run the Angular Frontend Developer Studio
Start the frontend development server on port `4200`:
```bash
make aad-fe-dev
```

#### 5. Execute Test Suites
- **Unit Tests**:
  ```bash
  make test
  ```
- **Robot Framework E2E Integration Tests**:
  ```bash
  make robot-test
  ```

---

## Make Command Reference Summary (`make help`)

| Target | Description |
| :--- | :--- |
| `make help` | Display available Make targets and descriptions |
| `make dev` / `make aad-be-dev` | Run Rust backend dev server with auto-port cleanup (`8080`/`8079`) |
| `make aad-be-watch` | Run Rust backend with `cargo watch` auto-recompilation |
| `make aad-be-migrate` | Run database migrations against PostgreSQL |
| `make aad-be-docker-run` | Build and run backend container locally |
| `make aad-fe-dev` | Run Angular frontend dev server (`4200`) |
| `make aad-fe-docker-run` | Build and run frontend container locally |
| `make db-up` / `make compose-db-up` | Start PostgreSQL container via Docker Compose |
| `make db-down` / `make compose-db-down` | Stop PostgreSQL container |
| `make compose-db-clean` | Stop PostgreSQL container and remove data volumes |
| `make stop-other-db` | Stop conflicting PostgreSQL containers (`sward-postgres`) |
| `make test` | Run Rust backend unit tests (`cargo test`) |
| `make build-be` / `make build-fe` | Build production Docker images |
| `make robot-test` | Execute Robot Framework integration test runner |
| `make garden-up` | Deploy local development environment via Garden |
