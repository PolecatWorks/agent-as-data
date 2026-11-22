# Agent-as-Data Keycloak Service Claims & Auth Addendum PRD

> [!IMPORTANT]
> **Canonical Auth Pattern**: Global Keycloak OIDC (`shell-ui` + `ecosystem-audiences`), `@polecatworks/mfe-shared` token distribution, zero-fake-auth rules, dynamic JWKS verification, and M2M `client_credentials` mechanics are defined canonically in [**Mfe-shell: Ecosystem Keycloak OIDC, Claims Distribution & Service Auth Pattern PRD**](https://github.com/PolecatWorks/Mfe-shell/blob/main/spec/prds/keycloak-oidc-and-claims-distribution-prd.md).
>
> This document specifies only the **Agent-as-Data (`aad-be-container`, `aad-fe-container`)** service-local contract.

---

## 1. Keycloak Client & Roles (`agent-as-data-api`)

- **Client ID (`aud`)**: `agent-as-data-api`
- **Type**: Bearer-only / Resource Server (declared in `fluxcd-dev/keycloak-crs.yaml`)
- **Client Roles** (`resource_access["agent-as-data-api"].roles`):
  - `agent:read`: View published agents, prompt templates, traits, knowledge nodes, and skills.
  - `agent:author`: Draft and test new agent, trait, tool, and skill definitions.
  - `agent:publish`: Promote agents and skills to the production/enterprise registry.
  - `governance:admin`: Set compliance rules, manage access policies, and audit agent telemetry.

---

## 2. Service-Specific Token Claims (`agent_as_data`)

```json
{
  "sub": "4a761e82-2d17-4861-8290-79883bfad211",
  "aud": ["agent-as-data-api"],
  "resource_access": {
    "agent-as-data-api": {
      "roles": ["agent:read", "agent:author", "agent:publish"]
    }
  },
  "agent_as_data": {
    "organization_id": "polecatworks-engineering",
    "publishing_tier": "verified-author",
    "allowed_registries": ["enterprise-core", "community"]
  }
}
```

---

## 3. Service-Local Integration Notes

- **Backend (`aad-be-container`, Rust/Axum)**: Protects all REST (`/api/aad/v1/*`) and unified MCP (`/mcp`, `/api/aad/v1/mcp`) endpoints with the canonical JWKS middleware (`aud` contains `agent-as-data-api`), using `claims.sub` as the authenticated owner UUID.
- **Frontend (`aad-fe-container`)**: Consumes `SharedContextService.getAccessToken()` via `@polecatworks/mfe-shared` (replacing `ANONYMOUS_OWNER_ID` placeholders).
- **CLI & M2M (`agent-as-data-sa`)**: Used by `aad-be ctl`, automated CI/CD hydration pipelines, and outbound MCP calls to `graphrag-api` / `data-engine-api`.
