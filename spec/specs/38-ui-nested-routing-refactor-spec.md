# Spec 38: UI Nested Routing Refactor

## Status
`complete`

## Reference PRD
[UI Nested Routing Refactor PRD](../prds/ui-nested-routing-refactor-prd.md)
[Agent UI PRD](../prds/agent-ui-testing-kit-prd.md)

## Objective
Decouple the existing monolithic registry components (Agents, Skills, Traits, Tools) into a standardized nested Master-Detail routing architecture. This involves extracting the layout and sidebar into a parent layout component and delegating the detail view to child route components rendered within a `<router-outlet>`.

## Requirements

### 1. Route Reconfiguration
The `app.routes.ts` file must be updated to use nested `children` routes for each entity domain:
```typescript
{
  path: 'agents',
  loadComponent: () => import('./components/agent-registry/agent-registry-layout.component').then(m => m.AgentRegistryLayoutComponent),
  children: [
    { path: '', loadComponent: () => import('./components/agent-registry/agent-empty.component').then(m => m.AgentEmptyComponent), pathMatch: 'full' },
    { path: ':id', loadComponent: () => import('./components/agent-registry/agent-detail.component').then(m => m.AgentDetailComponent) }
  ]
}
```
*(Repeat pattern for `skills`, `traits`, and `tools`)*

### 2. Layout Components (`*-layout.component`)
- **Role**: Provides the global shell and master list.
- **Renders**: `<app-top-navbar>`, `<app-entity-sidebar-list>`, and a `<router-outlet>` in the main content area.
- **State**: Manages the fetching of the master list (e.g. `skills`, `agents`) and passes it to the sidebar list.
- **Routing**: Sidebar clicks must trigger a `router.navigate(['/entities', id])` instead of mutating internal selection state.

### 3. Detail Components (`*-detail.component`)
- **Role**: Renders the specific details, forms, and attachments for a single entity.
- **State**: Must subscribe to `ActivatedRoute.paramMap` to detect changes to the `:id` parameter and fetch the corresponding entity data.
- **Isolation**: Must not contain the sidebar or top navbar.

### 4. Empty State Components (`*-empty.component`)
- **Role**: Renders a placeholder when no entity is selected (index route).
- **Renders**: Uses the shared `<app-empty-state>` component with a message instructing the user to select an entity from the sidebar or create a new one.

## Implementation Sequence
1. **Agent Registry**: Refactor into `AgentRegistryLayoutComponent`, `AgentDetailComponent`, and `AgentEmptyComponent`. Update routing.
2. **Skills Registry**: Refactor into `SkillsRegistryLayoutComponent`, `SkillDetailComponent`, and `SkillEmptyComponent`. Update routing.
3. **Traits Registry**: Refactor into `TraitsRegistryLayoutComponent`, `TraitDetailComponent`, and `TraitEmptyComponent`. Update routing.
4. **Tool Manager**: Refactor into `ToolManagerLayoutComponent`, `ToolDetailComponent`, and `ToolEmptyComponent`. Update routing.

## Test Strategy
### Unit Tests
- `AgentDetailComponent`: Verify subscription to `ActivatedRoute.paramMap` triggers `loadAgent(id)`.
- `AgentRegistryLayoutComponent`: Verify clicking an item in the sidebar calls `Router.navigate`.

### Integration Tests (Playwright / UI Automation)
- **Deep Linking**: Verify that navigating directly to `/agents/<uuid>` renders the detail pane correctly and highlights the active item in the sidebar.
- **Navigation Lifecycle**: Verify that clicking different items in the sidebar updates the URL without causing a full page refresh.
