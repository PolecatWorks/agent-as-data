---
status: draft
---

# 29. Refactoring Lab UI Enhancement Spec

## Objective
Improve the UI of the Agent Refactoring & Compression Lab (`/refactoring-lab`) by replacing raw agent IDs with human-readable names, adding hover-over descriptions, and enabling click-to-navigate functionality.

## Implementation Details
**Location:** `aad-fe-container/src/app/pages/refactoring-lab/` (or wherever the refactoring lab component is located).

1. **Entity Hydration & Presentation**:
   - The UI currently renders raw UUIDs returned from the cluster analysis (`POST /api/v1/agents/refactor/analyze`).
   - If the backend analysis payload does not include the entity `name` and `description`, the frontend must either:
     a) Hydrate the entities by joining with the cached agent list.
     b) Modify the backend payload to return `name` and `description` alongside the ID in the cluster groupings.
2. **Hover Tooltips**:
   - Implement an Angular Material Tooltip (`matTooltip`) on the rendered agent name.
   - Bind the tooltip to the agent's `description`.
3. **Navigation**:
   - Wrap the entity name in a router link or `(click)` handler.
   - Route to `/agents/:id` when the entity is clicked.

## Tests
- Ensure the refactoring lab renders successfully without throwing undefined property errors if an agent name is missing.
- Verify clicking the entity name successfully pushes the router state to the agent details view.
