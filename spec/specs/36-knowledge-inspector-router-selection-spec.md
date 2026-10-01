# Spec 36: Knowledge Inspector Router State Synchronization & Deep Linking

## Status: `complete`

## Primary PRD References
- [Knowledge & Data System PRD](../prds/knowledge-data-system-prd.md)
- [Agent UI & Testing Kit PRD](../prds/agent-ui-testing-kit-prd.md)
- Extends: [29-knowledge-registry-ui-bread-spec.md](./29-knowledge-registry-ui-bread-spec.md) & [35-knowledge-markdown-document-retention-spec.md](./35-knowledge-markdown-document-retention-spec.md)

---

## 1. Problem Statement & Architecture Goals

In previous specs ([Spec 29](./29-knowledge-registry-ui-bread-spec.md) and [Spec 35](./35-knowledge-markdown-document-retention-spec.md)), the Knowledge Inspector UI (`/knowledge-inspector`) provides a dual-pane layout for managing knowledge nodes, inspecting graph tuples, and tracking source document provenance.

However, unlike the other registries in the platform (`/agents/:id`, `/traits/:id`, `/skills/:id`, `/tools/:id`, `/workbench/:benchId/:threadId`):
1. **No Deep Linking**: The route configuration only defines `/knowledge-inspector`. There is no parameterized route `/knowledge-inspector/:id`.
2. **Missing Router Sync on Selection**: Selecting an item from the sidebar list, clicking a derived concept link, or clicking a source document badge updates internal component state (`selectedNode`) in-memory, but does not update the browser URL bar.
3. **Broken Reload & History Navigation**: If a user refreshes the page, shares a link, or clicks the browser's Back/Forward buttons, the active selection is lost, returning the user to the unselected dashboard state.

### Objectives of this Specification
- **Route Definition**: Register `/knowledge-inspector/:id` in `app.routes.ts` mapping to `KnowledgeInspectorComponent`.
- **Bidirectional Router State Synchronization**:
  - Selecting any knowledge node updates the browser URL to `/knowledge-inspector/:id`.
  - Direct navigation to `/knowledge-inspector/:id` (via URL entry, bookmark, or browser history back/forward) reads the `:id` parameter, resolves the node, highlights it in the sidebar, and hydrates the right workspace view.
  - Creating a new item or clearing selection synchronizes the route back to `/knowledge-inspector`.
- **Provenance & Graph Navigation Parity**:
  - Clicking on a source document badge or a derived concept link uses the Angular Router to navigate to `/knowledge-inspector/:id`.
- **Strict TDD Implementation**: Write comprehensive unit tests in `knowledge-inspector.component.spec.ts` validating router navigation on selection, route param resolution on initialization, and DOM active state reflection.

---

## 2. Technical Architecture & Component Design

```mermaid
flowchart TD
    subgraph BrowserRoute["Angular Router Navigation"]
        RouteBase["/knowledge-inspector"]
        RouteParam["/knowledge-inspector/:id"]
    end

    subgraph KnowledgeInspectorComponent["KnowledgeInspectorComponent"]
        ParamSub["route.paramMap Subscription"]
        SelectAction["selectNode(node) / navigateToNodeById(id)"]
        CreateAction["createNewNode()"]
        NodeList["nodes: KnowledgeNode[]"]
        Workspace["selectedNode / nodeForm / derivedConcepts"]
    end

    RouteParam -->|paramMap emits :id| ParamSub
    ParamSub -->|Find in nodes or fetch| Workspace
    SelectAction -->|router.navigate(['/knowledge-inspector', id])| RouteParam
    CreateAction -->|router.navigate(['/knowledge-inspector'])| RouteBase
    Workspace -->|Reflect in View| DOM["Active Sidebar Highlight + Inspector View"]
```

### Route Configuration (`aad-fe-container/src/app/app.routes.ts`)
Add route for `/knowledge-inspector/:id`:
```typescript
{
  path: 'knowledge-inspector',
  loadComponent: () =>
    import('./components/knowledge-inspector/knowledge-inspector.component').then(
      (m) => m.KnowledgeInspectorComponent
    ),
},
{
  path: 'knowledge-inspector/:id',
  loadComponent: () =>
    import('./components/knowledge-inspector/knowledge-inspector.component').then(
      (m) => m.KnowledgeInspectorComponent
    ),
},
```

### Component State Synchronization (`knowledge-inspector.component.ts`)
1. **Dependency Injection**: Inject `ActivatedRoute` and `Router`.
2. **Route Parameter Listener**:
   ```typescript
   this.route.paramMap.subscribe((params) => {
     const id = params.get('id');
     if (id) {
       this.handleRouteSelection(id);
     } else {
       // Clear selection if not creating new
       if (!this.isEditing) {
         this.selectedNode = null;
       }
     }
   });
   ```
3. **Selection Handler**:
   ```typescript
   selectNode(node: KnowledgeNode, triggerNavigation: boolean = true) {
     this.selectedNode = node;
     this.isEditing = false;
     this.showDeleteConfirm = false;
     this.showMarkdownImport = false;
     this.derivedConcepts = [];

     if (triggerNavigation) {
       this.router.navigate(['/knowledge-inspector', node.id]);
     }
     // Hydrate tuples & derived concepts...
   }
   ```
4. **Navigation by ID**:
   ```typescript
   navigateToNodeById(nodeId: string): void {
     if (!nodeId) return;
     this.router.navigate(['/knowledge-inspector', nodeId]);
   }
   ```
5. **New Node / Deselect**:
   ```typescript
   createNewNode() {
     this.selectedNode = null;
     this.isEditing = true;
     this.router.navigate(['/knowledge-inspector']);
     // Reset nodeForm...
   }
   ```

---

## 3. Test Strategy & TDD Verification

### Unit Tests (`aad-fe-container/src/app/components/knowledge-inspector/knowledge-inspector.component.spec.ts`)
1. **Route Param Resolution on Init**: Verify that when `ActivatedRoute.paramMap` emits an `id`, the component finds the matching node from `nodes` (or calls `apiService.getKnowledgeNode`) and sets `selectedNode`.
2. **Selection Triggers Router Navigation**: Verify that `selectNode(node)` triggers `router.navigate(['/knowledge-inspector', node.id])`.
3. **`navigateToNodeById` Triggers Router Navigation**: Verify that `navigateToNodeById('target-id')` triggers `router.navigate(['/knowledge-inspector', 'target-id'])`.
4. **`createNewNode` Resets Route**: Verify that `createNewNode()` triggers `router.navigate(['/knowledge-inspector'])`.
5. **DOM Reflection**: Verify that the selected item in the sidebar receives the active highlight CSS class and the workspace displays the node details.

---

## 4. Acceptance Criteria
- [ ] Direct URL navigation to `/knowledge-inspector/<node_id>` loads the node and reflects selection in both the sidebar and workspace.
- [ ] Clicking any node in the sidebar list updates the browser URL to `/knowledge-inspector/<node_id>`.
- [ ] Clicking a derived concept link or source document chip routes to `/knowledge-inspector/<target_id>`.
- [ ] Clicking "+ New Knowledge" resets selection and updates the route to `/knowledge-inspector`.
- [ ] All unit tests pass in `aad-fe-container`.
