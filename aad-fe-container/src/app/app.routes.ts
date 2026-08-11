import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', redirectTo: 'agent-registry', pathMatch: 'full' },
  { path: 'agent-registry', loadComponent: () => import('./components/agent-registry/agent-registry.component').then(m => m.AgentRegistryComponent) },
  { path: 'agent-registry/:id', loadComponent: () => import('./components/agent-registry/agent-registry.component').then(m => m.AgentRegistryComponent) },

  { path: 'interactive-testing', loadComponent: () => import('./components/interactive-testing/interactive-testing.component').then(m => m.InteractiveTestingComponent) },
  { path: 'network-visualizer', loadComponent: () => import('./components/network-visualizer/network-visualizer.component').then(m => m.NetworkVisualizerComponent) },
  { path: 'refactoring-lab', loadComponent: () => import('./components/refactoring-lab/refactoring-lab.component').then(m => m.RefactoringLabComponent) },
  { path: 'knowledge-inspector', loadComponent: () => import('./components/knowledge-inspector/knowledge-inspector.component').then(m => m.KnowledgeInspectorComponent) },
  { path: 'mcp-manager', loadComponent: () => import('./components/mcp-manager/mcp-manager.component').then(m => m.McpManagerComponent) },
];
