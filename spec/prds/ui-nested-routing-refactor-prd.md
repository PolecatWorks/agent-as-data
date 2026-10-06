# PRD: UI Nested Routing Refactor

## Objective
To decouple large monolithic registry pages into standard Master-Detail nested routing architectures using Angular's `<router-outlet>`. This ensures deep-linking works perfectly, isolates component states, prevents massive single-file templates, and adheres to Angular best practices.

## Current State
Currently, pages like Agent Registry, Skills Registry, Tool Manager, and Traits Registry render both the sidebar list and the detailed entity view within a single component. Selection state (`selectedAgent`, `selectedSkill`, etc.) is tracked manually inside the component logic, and the UI re-renders conditionally based on this internal state.

## Proposed Nested Routing Architecture
1. **Layout Wrapper Component**:
   - Mapped to the base route (e.g., `/agents`).
   - Renders the global layout (Top Navbar) and the Master view (Entity Sidebar List).
   - Contains a `<router-outlet>` in the main content area.
2. **Index / Empty State Component**:
   - Mapped to the base route (`/agents` with `pathMatch: 'full'`).
   - Renders an empty state encouraging the user to select an entity from the sidebar or create a new one.
3. **Detail Component**:
   - Mapped to the child route (e.g., `/agents/:id`).
   - Renders the specific details and edit forms for the selected entity.
   - Subscribes to the `ActivatedRoute` param map to load the correct entity on initialization.

## Entities to Refactor
- Agent Registry (`/agents`, `/agents/:id`)
- Skills Registry (`/skills`, `/skills/:id`)
- Traits Registry (`/traits`, `/traits/:id`)
- Tool Manager (`/tools`, `/tools/:id`)

## Dependencies
- Must utilize the recently developed `app-empty-state` shared component.
