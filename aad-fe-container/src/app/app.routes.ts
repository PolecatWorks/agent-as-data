import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: 'knowledge-explorer', loadComponent: () => import('./components/knowledge-explorer/knowledge-explorer.component').then(m => m.KnowledgeExplorerComponent) },
  { path: 'semantic-search', loadComponent: () => import('./pages/semantic-search/semantic-search.component').then(m => m.SemanticSearchComponent) },
  { path: '', redirectTo: 'home', pathMatch: 'full' },
  { path: 'home', loadComponent: () => import('./components/home/home.component').then(m => m.HomeComponent) },
  { path: 'detail', loadComponent: () => import('./components/detail/detail.component').then(m => m.DetailComponent) },
  { 
    path: 'agents', 
    loadComponent: () => import('./components/agent-registry/agent-registry-layout/agent-registry-layout.component').then(m => m.AgentRegistryLayoutComponent),
    children: [
      { path: '', loadComponent: () => import('./components/agent-registry/agent-empty/agent-empty.component').then(m => m.AgentEmptyComponent), pathMatch: 'full' },
      { path: 'new', loadComponent: () => import('./components/agent-registry/agent-detail.component').then(m => m.AgentDetailComponent) },
      { path: ':id', loadComponent: () => import('./components/agent-registry/agent-detail.component').then(m => m.AgentDetailComponent) }
    ]
  },
  { path: 'traits', loadComponent: () => import('./components/traits-registry/traits-registry.component').then(m => m.TraitsRegistryComponent) },
  { path: 'traits/:id', loadComponent: () => import('./components/traits-registry/traits-registry.component').then(m => m.TraitsRegistryComponent) },
  {
    path: 'skills',
    loadComponent: () => import('./components/skills-registry/skills-registry-layout/skills-registry-layout.component').then(m => m.SkillsRegistryLayoutComponent),
    children: [
      { path: '', loadComponent: () => import('./components/skills-registry/skill-empty/skill-empty.component').then(m => m.SkillEmptyComponent), pathMatch: 'full' },
      { path: 'new', loadComponent: () => import('./components/skills-registry/skill-detail.component').then(m => m.SkillDetailComponent) },
      { path: ':id', loadComponent: () => import('./components/skills-registry/skill-detail.component').then(m => m.SkillDetailComponent) }
    ]
  },

  { path: 'interactive-testing', loadComponent: () => import('./components/interactive-testing/interactive-testing.component').then(m => m.InteractiveTestingComponent) },
  { path: 'network-visualizer', loadComponent: () => import('./components/network-visualizer/network-visualizer.component').then(m => m.NetworkVisualizerComponent) },
  { path: 'network-visualizer/:type/:id', loadComponent: () => import('./components/network-visualizer/network-visualizer.component').then(m => m.NetworkVisualizerComponent) },
  { path: 'refactoring-lab', loadComponent: () => import('./components/refactoring-lab/refactoring-lab.component').then(m => m.RefactoringLabComponent) },
  { path: 'knowledge-inspector', loadComponent: () => import('./components/knowledge-inspector/knowledge-inspector.component').then(m => m.KnowledgeInspectorComponent) },
  { path: 'knowledge-inspector/:id', loadComponent: () => import('./components/knowledge-inspector/knowledge-inspector.component').then(m => m.KnowledgeInspectorComponent) },
  { path: 'tools', loadComponent: () => import('./components/tool-manager/tool-manager.component').then(m => m.ToolManagerComponent) },
  { path: 'tools/:id', loadComponent: () => import('./components/tool-manager/tool-manager.component').then(m => m.ToolManagerComponent) },
  { path: 'workbench', loadComponent: () => import('./components/workbench/workbench.component').then(m => m.WorkbenchComponent) },
  { path: 'workbench/:benchId', loadComponent: () => import('./components/workbench/workbench.component').then(m => m.WorkbenchComponent) },
  { path: 'workbench/:benchId/:threadId', loadComponent: () => import('./components/workbench/workbench.component').then(m => m.WorkbenchComponent) },
  { path: 'agent-context', loadComponent: () => import('./components/agent-context/agent-context.component').then(m => m.AgentContextComponent) },
];

