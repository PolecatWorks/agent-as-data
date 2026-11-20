# Agent-as-Data Keycloak Client Claims, Governance & Registry Security PRD

## Overview
This PRD defines the authentication, authorization, and claims management architecture for **Agent-as-Data (`agent-as-data`, `mcp-server-container`, `agent-ui-testing-kit`)**.

It specifies:
1. **Resource Server Protection (`agent-as-data-api`)**: Enforcing OIDC token verification via Keycloak JWKS on all registry APIs, skill definitions, and MCP endpoints.
2. **Domain-Specific Governance Claims**: Controlling access to agent versioning, skill promotion, and execution telemetry through `resource_access['agent-as-data-api']`.
3. **Machine-to-Machine Integration (`agent-as-data-sa`)**: Facilitating trusted communications between orchestrators, MCP servers, and background synchronizers.
4. **Zero Fake/Dev Auth Policy**: Enforcing strict fail-closed identity evaluation with zero reliance on client-supplied user headers.

---

## Architecture Topology

```mermaid
flowchart TD
    subgraph Consumers ["Consumers"]
        PortalUI["MFE Shell Studio View (agent-as-data UI)"]
        Orchestrator["Angry-Agent / Other LLM Agents (SA Token)"]
    end

    subgraph Identity ["Keycloak (auth.svc)"]
        JWKS["JWKS Discovery & Certs"]
        Client_API["KeycloakClient: agent-as-data-api\n(Bearer-Only)"]
        Client_SA["KeycloakClient: agent-as-data-sa\n(Confidential / Service Account)"]
    end

    subgraph Service ["Agent-as-Data Services"]
        AuthMiddleware["FastAPI / Rust Auth Middleware"]
        AgentRegistry["Agent & Skill Registry"]
        MCPRouter["MCP Server Endpoints"]
        GovernanceEngine["Governance, Versioning & Audit"]
    end

    Consumers -->|Bearer Token (aud: agent-as-data-api)| AuthMiddleware
    AuthMiddleware -->|Validate via JWKS| JWKS
    AuthMiddleware -->|Authenticated UserContext| AgentRegistry & MCPRouter & GovernanceEngine
    GovernanceEngine -->|Service token calls| Keycloak
```

---

## Specification Details

### 1. Keycloak Client Definition (`KeycloakClient: agent-as-data-api`)
- **Client ID**: `agent-as-data-api`
- **Type**: Bearer-only / Resource Server
- **Audience**: `aud: ["agent-as-data-api"]`
- **Client Roles**:
  - `agent:read`: View published agents, prompt templates, and skills.
  - `agent:author`: Draft and test new agent and skill definitions.
  - `agent:publish`: Promote agents to production/enterprise registry.
  - `governance:admin`: Set compliance rules, manage access policies, audit agent telemetry.

### 2. Unique Token Claims for Agent-as-Data
```json
{
  "iss": "http://keycloak-keycloakx-http.auth.svc.cluster.local/auth/realms/sw-dev",
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

### 3. Service Account Integration (`agent-as-data-sa`)
- Used by automated CI/CD pipelines, MCP daemon runners, and telemetry collectors to push and pull agent definitions programmatically.
