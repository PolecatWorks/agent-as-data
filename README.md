# Agent-As-Data (AAD)

Agent-As-Data is a declarative platform and specification for representing AI agents as structured, queryable data in PostgreSQL rather than imperative code.

## Structure
- `aad-be-container/`: Rust backend microservice using Axum, SQLx, Figment, and Tokio.
- `spec/`: Specifications and PRDs defining structured Agent data models and guardrails.
- `docker-compose/`: Docker Compose setup for local PostgreSQL.
- `charts/agent-as-data/`: Helm deployment chart.
- `fluxcd-dev/`: FluxCD Kustomization / HelmRelease dev manifests.
- `garden.yml`: Garden configuration for K8s environment management.
- `.github/workflows/`: CI workflows for Rust testing and Helm linting.

## Quickstart
```bash
make db-up   # Start PostgreSQL container
make dev     # Run backend dev server
```
