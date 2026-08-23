import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: '', redirectTo: 'home', pathMatch: 'full' },
  { path: 'home', loadComponent: () => import('./components/home/home.component').then(m => m.HomeComponent) },
  { path: 'agent-registry', loadComponent: () => import('./components/agent-registry/agent-registry.component').then(m => m.AgentRegistryComponent) },
  { path: 'agent-registry/:id', loadComponent: () => import('./components/agent-registry/agent-registry.component').then(m => m.AgentRegistryComponent) },
  { path: 'traits-registry', loadComponent: () => import('./components/traits-registry/traits-registry.component').then(m => m.TraitsRegistryComponent) },
  { path: 'traits-registry/:id', loadComponent: () => import('./components/traits-registry/traits-registry.component').then(m => m.TraitsRegistryComponent) },
  { path: 'skills-registry', loadComponent: () => import('./components/skills-registry/skills-registry.component').then(m => m.SkillsRegistryComponent) },
  { path: 'skills-registry/:id', loadComponent: () => import('./components/skills-registry/skills-registry.component').then(m => m.SkillsRegistryComponent) },

  { path: 'interactive-testing', loadComponent: () => import('./components/interactive-testing/interactive-testing.component').then(m => m.InteractiveTestingComponent) },
  { path: 'network-visualizer', loadComponent: () => import('./components/network-visualizer/network-visualizer.component').then(m => m.NetworkVisualizerComponent) },
  { path: 'network-visualizer/:type/:id', loadComponent: () => import('./components/network-visualizer/network-visualizer.component').then(m => m.NetworkVisualizerComponent) },
  { path: 'refactoring-lab', loadComponent: () => import('./components/refactoring-lab/refactoring-lab.component').then(m => m.RefactoringLabComponent) },
  { path: 'knowledge-inspector', loadComponent: () => import('./components/knowledge-inspector/knowledge-inspector.component').then(m => m.KnowledgeInspectorComponent) },
  { path: 'tools', loadComponent: () => import('./components/mcp-manager/mcp-manager.component').then(m => m.McpManagerComponent) },
  { path: 'tools/:id', loadComponent: () => import('./components/mcp-manager/mcp-manager.component').then(m => m.McpManagerComponent) },
];

