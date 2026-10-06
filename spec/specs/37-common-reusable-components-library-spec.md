# Spec 37: Common Reusable Components Library

## Status
`draft`

## Reference PRD
[Agent UI PRD](../prds/agent-ui-testing-kit-prd.md)

## Objective
To ensure maximum maintainability and visual consistency, the frontend application (`aad-fe-container`) MUST implement a standard set of reusable Angular Standalone Components. These components form the foundation of all application views, eliminating redundant code and standardizing layout behaviors, interactions, and theming.

## Expected Operational Outcomes

### 1. Global Top Navbar (`<app-top-navbar>`)
- **Outcome**: A single, parameterized component (`h-14 bg-white border-b border-slate-200 shadow-sm`) is used across all primary screens.
- **Contract**:
  - Inputs: `@Input() workspaceTitle: string`, `@Input() workspaceIcon: string`, `@Input() conceptGuideConfig: ConceptGuideConfig | null`.
  - Encapsulates the hamburger app menu, click-to-switch workspace dropdown, layout toggles, and user profile avatar.
  - Mounts `<app-concept-guide>` internally when `conceptGuideConfig` is provided.

### 2. Entity Sidebar List (`<app-entity-sidebar-list>`)
- **Outcome**: A collapsible, independently scrolling left-hand panel (`w-72` expanded / `w-16` collapsed) is implemented for lists of entities.
- **Contract**:
  - Handles its own search/filter input (`@Output() searchChange: EventEmitter<string>`).
  - Implements virtual scrolling for performance.
  - Exposes an `<ng-content>` slot or uses `ngTemplateOutlet` for custom list item rendering.

### 3. Standard Entity Card (`<app-entity-card>`)
- **Outcome**: A unified card component representing domain entities.
- **Contract**:
  - Inputs: `@Input() icon: string`, `@Input() title: string`, `@Input() version: string`, `@Input() description: string`, `@Input() badges: {label: string, icon?: string}[]`.
  - Outputs: `@Output() cardClick: EventEmitter<void>`.
  - Consistent hover and focus states.

### 4. Zero-Footprint Concept Guide (`<app-concept-guide>`)
- **Outcome**: Interactive popover trigger pill used for domain education without consuming active working area.
- **Contract**:
  - Features 200ms debounce hover, pinned click state.
  - Accepts typed configuration for analogies, tab mappings, and architecture deep-link routing.

### 5. Detail Form Action Bar (`<app-detail-action-bar>`)
- **Outcome**: An action bar positioned in the header of entity detail/edit views.
- **Contract**:
  - Standardized action buttons (Edit, Delete, Promote, Sync).
  - Built-in confirmation dialogs for destructive actions.
  - Outputs: `@Output() actionTriggered: EventEmitter<ActionType>`.

### 6. Terminal Console / Stream Viewer (`<app-terminal-console>`)
- **Outcome**: A dark-themed (`bg-slate-950`) console window for real-time SSE token streaming, execution logs, and output rendering.
- **Contract**:
  - Auto-scrolling on new input.
  - Syntax highlighting and status badge integration.
  - Copy-to-clipboard functionality.

### 7. Empty State & Error Canvas (`<app-empty-state>`)
- **Outcome**: Consistent messaging when lists are empty, searches yield no results, or errors occur.
- **Contract**:
  - Inputs: `@Input() title: string`, `@Input() message: string`, `@Input() icon: string`, `@Input() actionLabel: string`.
  - Outputs: `@Output() actionClick: EventEmitter<void>`.

### 8. Entity Attachment Manager (`<app-entity-attachment-manager>`)
- **Outcome**: A single, encapsulated component for managing the attachment of associated entities (e.g., Skills to Agents, Traits to Skills).
- **Contract**:
  - Inputs: `@Input() title: string`, `@Input() themeColor: string` (e.g., `'indigo'`, `'emerald'`), `@Input() availableEntities: {id: string, name: string, description: string}[]`, `@Input() attachedIds: string[]`, `@Input() isEditing: boolean`.
  - Outputs: `@Output() attach: EventEmitter<string>`, `@Output() detach: EventEmitter<string>`.
  - Internally manages search/filter state for the available entities list.

## Test Strategy

### Unit Tests (Jasmine/Karma)
- **Component Rendering**: Verify each component renders its inputs correctly (e.g., `app-top-navbar` shows the correct title and icon).
- **Event Emitters**: Verify that clicks on cards, buttons in the action bar, and the empty state trigger the correct output events.
- **Concept Guide State**: Test the hover debounce and click-to-pin logic of `<app-concept-guide>`.
- **Terminal Auto-scroll**: Verify `<app-terminal-console>` scrolls to the bottom when new stream data is appended.
- **Entity Attachment Manager**: Verify search filtering logic correctly filters `availableEntities` and that `attach`/`detach` events emit the correct ID.

### Integration Tests (Playwright / UI Automation)
- **Visual Parity**: Verify that all screens using the `<app-top-navbar>` exhibit identical height, borders, and layout constraints.
- **Sidebar Collapse**: Test that `<app-entity-sidebar-list>` smoothly transitions between expanded (`w-72`) and collapsed (`w-16`) states.
- **Concept Guide Positioning**: Ensure `<app-concept-guide>` popovers do not push down page content and close when clicking outside.
- **Action Bar Dialogs**: End-to-end verification that clicking "Delete" on `<app-detail-action-bar>` opens a confirmation dialog, and confirming proceeds with the intended action.
- **Entity Attachment Flow**: Test end-to-end flow of searching for an entity, clicking to attach it, verifying it appears as a pill, and clicking the 'x' to detach it in edit mode.
