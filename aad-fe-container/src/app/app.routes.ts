import { Routes } from '@angular/router';

export const routes: Routes = [
  { path: 'knowledge-explorer', loadComponent: () => import('./pages/knowledge-explorer/knowledge-explorer.component').then(m => m.KnowledgeExplorerComponent) },
  { path: 'semantic-search', loadComponent: () => import('./pages/semantic-search/semantic-search.component').then(m => m.SemanticSearchComponent) },
  { path: '', redirectTo: 'home', pathMatch: 'full' },
  { path: 'home', loadComponent: () => import('./pages/home/home.component').then(m => m.HomeComponent) },
  { path: 'detail', loadComponent: () => import('./pages/detail/detail.component').then(m => m.DetailComponent) },
  { 
    path: 'agents', 
    loadComponent: () => import('./pages/agent-registry/agent-registry-layout/agent-registry-layout.component').then(m => m.AgentRegistryLayoutComponent),
    children: [
      { path: '', loadComponent: () => import('./pages/agent-registry/agent-empty/agent-empty.component').then(m => m.AgentEmptyComponent), pathMatch: 'full' },
      { path: 'new', loadComponent: () => import('./pages/agent-registry/agent-detail.component').then(m => m.AgentDetailComponent) },
      { path: ':id', loadComponent: () => import('./pages/agent-registry/agent-detail.component').then(m => m.AgentDetailComponent) }
    ]
  },
  {
    path: 'traits',
    loadComponent: () => import('./pages/traits-registry/traits-registry-layout/traits-registry-layout.component').then(m => m.TraitsRegistryLayoutComponent),
    children: [
      { path: '', loadComponent: () => import('./pages/traits-registry/trait-empty/trait-empty.component').then(m => m.TraitEmptyComponent), pathMatch: 'full' },
      { path: 'new', loadComponent: () => import('./pages/traits-registry/trait-detail.component').then(m => m.TraitDetailComponent) },
      { path: ':id', loadComponent: () => import('./pages/traits-registry/trait-detail.component').then(m => m.TraitDetailComponent) }
    ]
  },
  {
    path: 'skills',
    loadComponent: () => import('./pages/skills-registry/skills-registry-layout/skills-registry-layout.component').then(m => m.SkillsRegistryLayoutComponent),
    children: [
      { path: '', loadComponent: () => import('./pages/skills-registry/skill-empty/skill-empty.component').then(m => m.SkillEmptyComponent), pathMatch: 'full' },
      { path: 'new', loadComponent: () => import('./pages/skills-registry/skill-detail.component').then(m => m.SkillDetailComponent) },
      { path: ':id', loadComponent: () => import('./pages/skills-registry/skill-detail.component').then(m => m.SkillDetailComponent) }
    ]
  },

  { path: 'interactive-testing', loadComponent: () => import('./pages/interactive-testing/interactive-testing.component').then(m => m.InteractiveTestingComponent) },
  { path: 'network-visualizer', loadComponent: () => import('./pages/network-visualizer/network-visualizer.component').then(m => m.NetworkVisualizerComponent) },
  { path: 'network-visualizer/:type/:id', loadComponent: () => import('./pages/network-visualizer/network-visualizer.component').then(m => m.NetworkVisualizerComponent) },
  { path: 'refactoring-lab', loadComponent: () => import('./pages/refactoring-lab/refactoring-lab.component').then(m => m.RefactoringLabComponent) },
  { path: 'knowledge-inspector', loadComponent: () => import('./pages/knowledge-inspector/knowledge-inspector.component').then(m => m.KnowledgeInspectorComponent) },
  { path: 'knowledge-inspector/:id', loadComponent: () => import('./pages/knowledge-inspector/knowledge-inspector.component').then(m => m.KnowledgeInspectorComponent) },
  { path: 'tools', loadComponent: () => import('./pages/tool-manager/tool-manager.component').then(m => m.ToolManagerComponent) },
  { path: 'tools/:id', loadComponent: () => import('./pages/tool-manager/tool-manager.component').then(m => m.ToolManagerComponent) },
  { path: 'workbench', loadComponent: () => import('./pages/workbench/workbench.component').then(m => m.WorkbenchComponent) },
  { path: 'workbench/:benchId', loadComponent: () => import('./pages/workbench/workbench.component').then(m => m.WorkbenchComponent) },
  { path: 'workbench/:benchId/:threadId', loadComponent: () => import('./pages/workbench/workbench.component').then(m => m.WorkbenchComponent) },
  { path: 'agent-context', loadComponent: () => import('./pages/agent-context/agent-context.component').then(m => m.AgentContextComponent) },
];

