import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';

import { APP_NAV_MENU_ITEMS, NavMenuItem } from '../../models/navigation';
export interface WorkspaceCard {
  id: string;
  title: string;
  category: 'Execution' | 'Registries' | 'Governance' | 'Knowledge';
  icon: string;
  description: string;
  path: string;
  actionText: string;
  colorClass: string;
  badge?: string;
  capabilities: string[];
}

export interface LifecyclePhase {
  step: number;
  title: string;
  subtitle: string;
  icon: string;
  accentClass: string;
  bgClass: string;
  borderClass: string;
  textClass: string;
  items: string[];
}

export interface TraitPillar {
  title: string;
  icon: string;
  badge: string;
  description: string;
  example: string;
}

export interface OperationalTenet {
  title: string;
  icon: string;
  description: string;
  guarantee: string;
}

@Component({
  selector: 'app-detail',
  standalone: true,
  imports: [
    CommonModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatMenuModule,
    RouterModule
  ],
  templateUrl: './detail.component.html',
  styleUrl: './detail.component.scss'
})
export class DetailComponent {
  menuItems: NavMenuItem[] = APP_NAV_MENU_ITEMS;

  lifecyclePhases: LifecyclePhase[] = [
    {
      step: 1,
      title: '1. Knowledge & Context',
      subtitle: 'Tacit Enterprise Ingestion',
      icon: 'library_books',
      accentClass: 'from-blue-500 to-indigo-600',
      bgClass: 'bg-blue-50/60',
      borderClass: 'border-blue-200',
      textClass: 'text-blue-900',
      items: [
        'Semantic RAG chunks (pgvector)',
        'SPO graph relation tuples',
        'Natural language context scoring'
      ]
    },
    {
      step: 2,
      title: '2. Specifications & Contracts',
      subtitle: 'Declarative Building Blocks',
      icon: 'verified',
      accentClass: 'from-emerald-500 to-teal-600',
      bgClass: 'bg-emerald-50/60',
      borderClass: 'border-emerald-200',
      textClass: 'text-emerald-900',
      items: [
        '3-Element Trait Contracts',
        'Skills Registry & JSON Schemas',
        'Declarative Agent Registry',
        'Remote Tools & MCP Ingestion'
      ]
    },
    {
      step: 3,
      title: '3. Governance & Topology',
      subtitle: 'Analysis & Optimization',
      icon: 'account_tree',
      accentClass: 'from-purple-500 to-violet-600',
      bgClass: 'bg-purple-50/60',
      borderClass: 'border-purple-200',
      textClass: 'text-purple-900',
      items: [
        'Mermaid delegation visualizer',
        'Duplication & cluster scan',
        'Intentional contradiction rules'
      ]
    },
    {
      step: 4,
      title: '4. Verification & Testing',
      subtitle: 'Continuous Quality Gates',
      icon: 'bug_report',
      accentClass: 'from-amber-500 to-orange-600',
      bgClass: 'bg-amber-50/60',
      borderClass: 'border-amber-200',
      textClass: 'text-amber-900',
      items: [
        'Real-time SSE token stream',
        'Local Rig / Ollama integration',
        'Contract verification tester'
      ]
    },
    {
      step: 5,
      title: '5. Workbench Execution',
      subtitle: 'Autonomous Project Runtimes',
      icon: 'chat',
      accentClass: 'from-rose-500 to-pink-600',
      bgClass: 'bg-rose-50/60',
      borderClass: 'border-rose-200',
      textClass: 'text-rose-900',
      items: [
        'Isolated bench filesystem',
        'Multi-turn Rig tool execution',
        'Bench working memory',
        'Distributed action cancellation'
      ]
    }
  ];

  traitPillars: TraitPillar[] = [
    {
      title: 'Capability Requirements',
      icon: 'build',
      badge: 'Required Tools & Permissions',
      description: 'The environmental access, system tools, or sensory inputs the agent must possess to satisfy this role.',
      example: 'e.g. AST parser, read_file, git_read_only'
    },
    {
      title: 'Behavioral Invariants',
      icon: 'gavel',
      badge: 'Unbreakable Constraints',
      description: 'Strict non-negotiable rules the agent MUST ALWAYS or MUST NEVER violate during reasoning or action.',
      example: 'e.g. MUST NEVER run untrusted binaries; MUST ALWAYS output valid JSON'
    },
    {
      title: 'Evaluation Rubrics',
      icon: 'fact_check',
      badge: 'Automated LLM Judge',
      description: 'Objective semantic criteria used by automated LLM-as-a-Judge evaluators to score compliance (0.0 - 1.0).',
      example: 'e.g. Pass threshold >= 0.85 on security vulnerability test suite'
    },
    {
      title: 'Inherited Baseline Guardrails',
      icon: 'shield',
      badge: 'Mandatory Interceptors',
      description: 'Pre-execution and post-execution guardrails automatically inherited by any agent implementing this trait.',
      example: 'e.g. PII regex filtering, API key redaction, prompt injection shields'
    }
  ];

  workspaceCards: WorkspaceCard[] = [
    {
      id: 'workbench',
      title: 'Workbench',
      category: 'Execution',
      icon: 'chat',
      description: 'Project-scoped bench workspaces with multi-turn conversation threads, shared filesystem tools, bench memory, and distributed action cancellation.',
      path: '/workbench',
      actionText: 'Open Workbench',
      colorClass: 'text-indigo-600 bg-indigo-50 border-indigo-200 hover:border-indigo-400',
      badge: 'Primary Execution',
      capabilities: ['Bench Filesystem Isolation', 'Rig Multi-Turn Tool Loop', 'Thread Runs & Cancellation']
    },
    {
      id: 'agents',
      title: 'Agent Registry & Builder',
      category: 'Registries',
      icon: 'smart_toy',
      description: 'Compose declarative agent blueprints, configure LLM reasoning models, attach execution guardrails, and track immutable revision lineage.',
      path: '/agents',
      actionText: 'Browse Agents',
      colorClass: 'text-blue-600 bg-blue-50 border-blue-200 hover:border-blue-400',
      badge: 'Declarative Spec',
      capabilities: ['Immutable agent_revisions', 'Guardrail Interceptors', 'Trait Contract Bindings']
    },
    {
      id: 'traits',
      title: 'Trait Contracts Registry',
      category: 'Registries',
      icon: 'verified',
      description: 'Inspect and define abstract behavioral contracts ("Interfaces for AI") establishing capabilities, invariants, rubrics, and inherited guardrails.',
      path: '/traits',
      actionText: 'Manage Traits',
      colorClass: 'text-emerald-600 bg-emerald-50 border-emerald-200 hover:border-emerald-400',
      badge: 'Abstract Contracts',
      capabilities: ['Decoupled Agent Delegation', 'Inherited Guardrails', 'Automated Verification']
    },
    {
      id: 'skills',
      title: 'Skills Registry',
      category: 'Registries',
      icon: 'extension',
      description: 'Define deterministic skill routines with strict input/output JSON schemas and promote skills to full reasoning agents with one click.',
      path: '/skills',
      actionText: 'Explore Skills',
      colorClass: 'text-teal-600 bg-teal-50 border-teal-200 hover:border-teal-400',
      badge: 'Deterministic Tools',
      capabilities: ['JSON Schema Validation', 'One-Click Promotion', 'Reusable Tool Attachment']
    },
    {
      id: 'tools',
      title: 'Remote Tools & MCP',
      category: 'Registries',
      icon: 'dns',
      description: 'Register external Model Context Protocol (MCP) servers over Stdio and SSE, browse cached argument schemas, and bind tools dynamically.',
      path: '/tools',
      actionText: 'Configure Tools',
      colorClass: 'text-cyan-600 bg-cyan-50 border-cyan-200 hover:border-cyan-400',
      badge: 'MCP Protocol',
      capabilities: ['Stdio & SSE Transports', 'Cached Argument Schemas', 'RAG Tool Discovery']
    },
    {
      id: 'interactive-testing',
      title: 'Interactive Testing Studio',
      category: 'Execution',
      icon: 'bug_report',
      description: 'Test agents and skills with real-time SSE token streaming, inspect system prompts and instructions, and test dynamic trait mapping overrides.',
      path: '/interactive-testing',
      actionText: 'Launch Studio',
      colorClass: 'text-amber-600 bg-amber-50 border-amber-200 hover:border-amber-400',
      badge: 'Live Playground',
      capabilities: ['Real-Time SSE Streaming', 'Live Prompt Inspector', 'Contract Fit Verification']
    },
    {
      id: 'network-visualizer',
      title: 'Delegation Network Graph',
      category: 'Governance',
      icon: 'account_tree',
      description: 'Visualize agent hierarchies, sub-agent delegation links, and skill dependencies in an interactive, filterable Mermaid network graph.',
      path: '/network-visualizer',
      actionText: 'View Network Graph',
      colorClass: 'text-purple-600 bg-purple-50 border-purple-200 hover:border-purple-400',
      badge: 'Topology Visualizer',
      capabilities: ['Interactive Mermaid Graph', 'Multi-Hop Delegation', 'Trait Interface Filters']
    },
    {
      id: 'refactoring-lab',
      title: 'Refactoring & Compression Lab',
      category: 'Governance',
      icon: 'build_circle',
      description: 'Scan agent embeddings for semantic overlap and duplication, harmonize conflicting instructions, and codify intentional persona contradictions.',
      path: '/refactoring-lab',
      actionText: 'Open Refactoring Lab',
      colorClass: 'text-violet-600 bg-violet-50 border-violet-200 hover:border-violet-400',
      badge: 'Optimization',
      capabilities: ['Cluster Overlap Scanning', 'Harmonization Diffing', 'Contradiction Management']
    },
    {
      id: 'knowledge-inspector',
      title: 'Knowledge Base Inspector',
      category: 'Knowledge',
      icon: 'library_books',
      description: 'Explore the dual-store Enterprise Brain: query semantic vector chunks (pgvector) and traverse Subject-Predicate-Object (SPO) relationship tuples.',
      path: '/knowledge-inspector',
      actionText: 'Query Knowledge',
      colorClass: 'text-sky-600 bg-sky-50 border-sky-200 hover:border-sky-400',
      badge: 'Enterprise Brain',
      capabilities: ['Hybrid Vector Search', 'SPO Graph Tuples', 'Multi-Hop Entity Traversal']
    },
    {
      id: 'agent-context',
      title: 'Agent Context Search',
      category: 'Knowledge',
      icon: 'search',
      description: 'Natural language task context search that evaluates separated multi-embeddings (name, description, prompt) to score and match relevant agents.',
      path: '/agent-context',
      actionText: 'Search Context',
      colorClass: 'text-slate-600 bg-slate-100 border-slate-300 hover:border-slate-500',
      badge: 'Semantic Scoring',
      capabilities: ['Multi-Embedding Scoring', 'Configurable Trace Depth', 'Semantic Fit Feedback']
    }
  ];

  operationalTenets: OperationalTenet[] = [
    {
      title: 'Zero Direct Runtime Env Vars',
      icon: 'settings_suggest',
      description: 'All configuration is loaded fail-fast at startup via AppConfig with strict schema validation. No unvetted process defaults.',
      guarantee: 'Fail-Fast Configuration'
    },
    {
      title: 'Deterministic Version Lineage',
      icon: 'history',
      description: 'Modifying an agent creates an immutable agent_revisions record. Executions and client bindings remain completely reproducible.',
      guarantee: 'Immutable History'
    },
    {
      title: 'Strict Entity Referencing',
      icon: 'link',
      description: 'Non-null database foreign keys prevent orphaned execution traces. Deletions archive entities safely to preserve referential integrity.',
      guarantee: 'Referential Integrity'
    },
    {
      title: 'Distributed Run Safety',
      icon: 'cancel',
      description: 'Multi-turn agent runs execute asynchronously with pre-tool cancellation hooks (thread_runs) ensuring immediate, safe run halts.',
      guarantee: 'Pre-Tool Cancellation'
    }
  ];
}
