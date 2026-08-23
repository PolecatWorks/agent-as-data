import { Component, ElementRef, OnInit, ViewChild, AfterViewInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { forkJoin, of } from 'rxjs';
import { catchError } from 'rxjs/operators';
import mermaid from 'mermaid';
import { ApiService, Agent, Skill, TraitContract } from '../../services/api.service';

export type EntityType = 'agent' | 'skill' | 'trait' | 'mcp';

export interface McpServer {
  id: string;
  server_name: string;
  transport_type: string;
  description?: string;
  tags?: string[];
  tools_count?: number;
}

export interface SelectableEntity {
  id: string;
  name: string;
  type: EntityType;
  version?: string;
  description?: string;
  tags?: string[];
}

@Component({
  selector: 'app-network-visualizer',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatTooltipModule,
    MatProgressSpinnerModule,
  ],
  templateUrl: './network-visualizer.component.html',
  styleUrl: './network-visualizer.component.scss'
})
export class NetworkVisualizerComponent implements OnInit, AfterViewInit {
  @ViewChild('mermaidContainer') mermaidContainer!: ElementRef;

  // Data
  agents: Agent[] = [];
  skills: Skill[] = [];
  traits: TraitContract[] = [];
  mcpServers: McpServer[] = [];

  // Selection
  allEntities: SelectableEntity[] = [];
  searchQuery: string = '';
  selectedEntity: SelectableEntity | null = null;

  // Trace depth
  traceDepth: number = 2;

  // State
  isLoading: boolean = false;
  renderError: string | null = null;
  private mermaidReady: boolean = false;
  private diagramCounter = 0;

  constructor(private apiService: ApiService, private route: ActivatedRoute, private router: Router) {}

  ngOnInit(): void {
    mermaid.initialize({
      startOnLoad: false,
      theme: 'default',
      flowchart: { htmlLabels: true, curve: 'basis' }
    });
    this.loadAllData();
  }

  ngAfterViewInit(): void {
    this.mermaidReady = true;
  }

  loadAllData(): void {
    this.isLoading = true;
    forkJoin({
      agents: this.apiService.getAgents().pipe(catchError(() => of([]))),
      skills: this.apiService.getSkills().pipe(catchError(() => of([]))),
      traits: this.apiService.getTraits().pipe(catchError(() => of({ ids: [] } as any))),
      mcpServers: this.apiService.getMcpServers().pipe(catchError(() => of([])))
    }).subscribe({
      next: ({ agents, skills, traits, mcpServers }) => {
        // The search endpoint returns sparse agent objects — hydrate each to its full record
        const sparseAgents = agents as Agent[];
        this.skills = skills as Skill[];
        this.mcpServers = (mcpServers as any[]).map(s => ({
          id: s.id,
          server_name: s.server_name,
          transport_type: s.transport_type,
          description: s.endpoint_config?.description || '',
          tags: s.endpoint_config?.tags || [],
          tools_count: s.cached_capabilities?.tools?.length || 0
        })) as McpServer[];
        const traitIds: string[] = (traits as any).ids || [];

        // Hydrate agents in parallel (full detail includes attached_skills, attached_agents, etc.)
        const agentHydrate$ = sparseAgents.length > 0
          ? forkJoin(sparseAgents.map(a =>
              this.apiService.getAgent(a.id).pipe(catchError(() => of(a)))
            ))
          : of([] as Agent[]);

        // Hydrate traits in parallel
        const traitHydrate$ = traitIds.length > 0
          ? forkJoin(traitIds.map(id =>
              this.apiService.getTrait(id).pipe(catchError(() => of(null)))
            ))
          : of([] as any[]);

        forkJoin({ fullAgents: agentHydrate$, fullTraits: traitHydrate$ }).subscribe({
          next: ({ fullAgents, fullTraits }) => {
            this.agents = (fullAgents as any[]).filter(Boolean) as Agent[];
            this.traits = (fullTraits as any[]).filter(Boolean) as TraitContract[];
            this.buildEntityList();
            this.isLoading = false;
            this.handleRouteParams();
          },
          error: () => {
            this.agents = sparseAgents;
            this.buildEntityList();
            this.isLoading = false;
            this.handleRouteParams();
          }
        });
      },
      error: () => {
        this.isLoading = false;
      }
    });
  }

  handleRouteParams(): void {
    this.route.paramMap.subscribe(params => {
      const type = params.get('type') as EntityType;
      const id = params.get('id');

      if (type && id) {
        let entityToSelect = this.allEntities.find(e => e.type === type && (e.id === id || e.name === id));
        if (entityToSelect) {
          this.selectedEntity = entityToSelect;
          this.renderError = null;
          setTimeout(() => this.buildAndRenderDiagram(), 50);
        }
      }
    });
  }

  buildEntityList(): void {
    this.allEntities = [
      ...this.agents.map(a => ({
        id: a.id,
        name: a.name,
        type: 'agent' as EntityType,
        version: a.current_version,
        description: a.description,
        tags: a.tags
      })),
      ...this.skills.map(s => ({
        id: s.id!,
        name: s.name,
        type: 'skill' as EntityType,
        version: s.current_version,
        description: s.description,
        tags: s.tags
      })),
      ...this.traits.map(t => ({
        id: t.id,
        name: t.name,
        type: 'trait' as EntityType,
        version: t.version,
        description: t.description,
        tags: t.tags
      })),
      ...this.mcpServers.map(m => ({
        id: m.id,
        name: m.server_name,
        type: 'mcp' as EntityType,
        version: '1.0.0',
        description: m.description,
        tags: m.tags
      }))
    ];
  }

  getFilteredEntities(): SelectableEntity[] {
    const q = this.searchQuery.toLowerCase().trim();
    if (!q) return this.allEntities;
    return this.allEntities.filter(e =>
      e.name.toLowerCase().includes(q) ||
      (e.description && e.description.toLowerCase().includes(q)) ||
      (e.tags && e.tags.some(t => t.toLowerCase().includes(q))) ||
      e.type.toLowerCase().includes(q)
    );
  }

  selectEntity(entity: SelectableEntity): void {
    this.router.navigate(['/network-visualizer', entity.type, entity.id]);
  }

  onDepthChange(): void {
    if (this.selectedEntity) {
      setTimeout(() => this.buildAndRenderDiagram(), 50);
    }
  }

  getEntityIcon(type: EntityType): string {
    switch (type) {
      case 'agent': return '🤖';
      case 'skill': return '⚡';
      case 'trait': return '🛡️';
      case 'mcp': return '🔌';
    }
  }

  getEntityTypeLabel(type: EntityType): string {
    switch (type) {
      case 'agent': return 'Agent';
      case 'skill': return 'Skill';
      case 'trait': return 'Trait';
      case 'mcp': return 'Tool';
    }
  }

  getEntityBadgeClass(type: EntityType): string {
    switch (type) {
      case 'agent': return 'badge-agent';
      case 'skill': return 'badge-skill';
      case 'trait': return 'badge-trait';
      case 'mcp': return 'badge-mcp';
    }
  }

  // ---- Mermaid generation ----

  buildAndRenderDiagram(): void {
    if (!this.selectedEntity) return;
    const code = this.buildMermaidCode(this.selectedEntity);
    this.renderDiagram(code);
  }

  private sanitizeId(name: string): string {
    return name.replace(/[^a-zA-Z0-9_]/g, '_');
  }

  private sanitizeLabel(label: string): string {
    return label.replace(/"/g, "'").replace(/[<>]/g, '');
  }

  private nodeIdFor(entityKey: string): string {
    return `N_${this.sanitizeId(entityKey)}`;
  }

  private agentEntityKey(id: string): string { return `agent:${id}`; }
  private skillEntityKey(id: string): string { return `skill:${id}`; }
  private traitEntityKey(name: string): string { return `trait:${name}`; }
  private mcpEntityKey(id: string): string { return `mcp:${id}`; }

  /**
   * Multi-depth graph traversal using BFS.
   * Follows all link types up to `this.traceDepth` hops from the root entity.
   */
  buildMermaidCode(entity: SelectableEntity): string {
    const lines: string[] = ['flowchart TD'];

    // Track rendered nodes and edges to avoid duplicates
    const visitedNodes = new Set<string>();
    const visitedEdges = new Set<string>();

    // node declarations queued up
    const nodeLines: string[] = [];
    const edgeLines: string[] = [];

    const emitNode = (key: string, label: string, style: string, isRoot: boolean, description: string = '') => {
      if (visitedNodes.has(key)) return;
      visitedNodes.add(key);
      const nid = this.nodeIdFor(key);
      const strokeWidth = isRoot ? '3px' : '1.5px';
      const styleWithRoot = isRoot ? style.replace('stroke-width:1.5px', `stroke-width:${strokeWidth}`) : style;
      nodeLines.push(`    ${nid}["${label}"]`);
      nodeLines.push(`    style ${nid} ${styleWithRoot}`);

      // Add click support for mermaid graph tooltip and link route
      let type = '';
      let id = '';
      if (key.startsWith('agent:')) {
        type = 'agent';
        id = key.slice('agent:'.length);
      } else if (key.startsWith('skill:')) {
        type = 'skill';
        id = key.slice('skill:'.length);
      } else if (key.startsWith('trait:')) {
        type = 'trait';
        const traitName = key.slice('trait:'.length);
        const matchedTrait = this.traits.find(t => t.name === traitName);
        id = matchedTrait ? matchedTrait.id : traitName;
      } else if (key.startsWith('mcp:')) {
        type = 'mcp';
        id = key.slice('mcp:'.length);
      }

      const linkUrl = `/network-visualizer/${type}/${id}`;
      // Clean up description for the mermaid tooltip syntax (remove quotes, newlines, etc.)
      const cleanDesc = description.replace(/"/g, "'").replace(/\n/g, ' ').substring(0, 100);
      nodeLines.push(`    click ${nid} href "${linkUrl}" "${cleanDesc}"`);
    };

    const emitEdge = (fromKey: string, toKey: string, label: string) => {
      const edgeKey = `${fromKey}-->${toKey}:${label}`;
      if (visitedEdges.has(edgeKey)) return;
      visitedEdges.add(edgeKey);
      edgeLines.push(`    ${this.nodeIdFor(fromKey)} -->|${label}| ${this.nodeIdFor(toKey)}`);
    };

    // Determine root entity key
    let rootKey: string;
    if (entity.type === 'agent') {
      rootKey = this.agentEntityKey(entity.id);
    } else if (entity.type === 'skill') {
      rootKey = this.skillEntityKey(entity.id);
    } else if (entity.type === 'mcp') {
      rootKey = this.mcpEntityKey(entity.id);
    } else {
      rootKey = this.traitEntityKey(entity.name);
    }

    // BFS queue: [entityKey, currentDepth]
    const queue: Array<[string, number]> = [[rootKey, 0]];
    const queued = new Set<string>([rootKey]);

    while (queue.length > 0) {
      const [currentKey, depth] = queue.shift()!;
      const isRoot = currentKey === rootKey;

      // Parse and emit the current node
      if (currentKey.startsWith('agent:')) {
        const agentId = currentKey.slice('agent:'.length);
        const agent = this.agents.find(a => a.id === agentId);
        const label = agent
          ? `🤖 ${this.sanitizeLabel(agent.name)}${isRoot ? ` v${agent.current_version}` : ''}`
          : `🤖 ${this.sanitizeLabel(agentId)}`;
        emitNode(currentKey, label,
          isRoot
            ? 'fill:#e0e7ff,stroke:#6366f1,stroke-width:3px,color:#1e1b4b'
            : 'fill:#e0e7ff,stroke:#6366f1,stroke-width:1.5px,color:#1e1b4b',
          isRoot, agent?.description || '');

        if (depth < this.traceDepth && agent) {
          // Forward: attached sub-agents
          (agent.attached_agents || []).forEach(subId => {
            const k = this.agentEntityKey(subId);
            emitEdge(currentKey, k, 'delegates');
            if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
          });
          // Forward: attached skills
          (agent.attached_skills || []).forEach(skillId => {
            const k = this.skillEntityKey(skillId);
            emitEdge(currentKey, k, 'uses skill');
            if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
          });
          // Forward: implements traits
          (agent.implements_traits || []).forEach(traitName => {
            const k = this.traitEntityKey(traitName);
            emitEdge(currentKey, k, 'implements');
            if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
          });
          // Forward: attached MCP servers
          (agent.attached_mcp_servers || []).forEach(mcpId => {
            const k = this.mcpEntityKey(mcpId);
            emitEdge(currentKey, k, 'uses MCP');
            if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
          });
          // Reverse: agents that delegate TO this agent
          this.agents
            .filter(a => (a.attached_agents || []).includes(agentId))
            .forEach(a => {
              const k = this.agentEntityKey(a.id);
              emitEdge(k, currentKey, 'delegates');
              if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
            });
        }

      } else if (currentKey.startsWith('skill:')) {
        const skillId = currentKey.slice('skill:'.length);
        const skill = this.skills.find(s => s.id === skillId);
        const label = skill
          ? `⚡ ${this.sanitizeLabel(skill.name)}${isRoot ? ` v${skill.current_version}` : ''}`
          : `⚡ ${this.sanitizeLabel(skillId)}`;
        emitNode(currentKey, label,
          isRoot
            ? 'fill:#fdf4ff,stroke:#a855f7,stroke-width:3px,color:#4c1d95'
            : 'fill:#fdf4ff,stroke:#a855f7,stroke-width:1.5px,color:#4c1d95',
          isRoot, skill?.description || '');

        if (depth < this.traceDepth && skill) {
          // Forward: sub-skills
          (skill.attached_skills || []).forEach(subId => {
            const k = this.skillEntityKey(subId);
            emitEdge(currentKey, k, 'uses skill');
            if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
          });
          // Forward: implements traits
          (skill.implements_traits || []).forEach(traitName => {
            const k = this.traitEntityKey(traitName);
            emitEdge(currentKey, k, 'implements');
            if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
          });
          // Forward: attached MCP servers
          (skill.attached_mcp_servers || []).forEach(mcpId => {
            const k = this.mcpEntityKey(mcpId);
            emitEdge(currentKey, k, 'uses MCP');
            if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
          });
          // Reverse: agents using this skill
          this.agents
            .filter(a => (a.attached_skills || []).includes(skillId))
            .forEach(a => {
              const k = this.agentEntityKey(a.id);
              emitEdge(k, currentKey, 'uses skill');
              if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
            });
          // Reverse: skills using this skill
          this.skills
            .filter(s => (s.attached_skills || []).includes(skillId))
            .forEach(s => {
              const k = this.skillEntityKey(s.id!);
              emitEdge(k, currentKey, 'uses skill');
              if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
            });
        }

      } else if (currentKey.startsWith('trait:')) {
        const traitName = currentKey.slice('trait:'.length);
        const trait = this.traits.find(t => t.name === traitName);
        const label = trait
          ? `🛡️ ${this.sanitizeLabel(traitName)}${isRoot ? ` v${trait.version}` : ''}`
          : `🛡️ ${this.sanitizeLabel(traitName)}`;
        emitNode(currentKey, label,
          isRoot
            ? 'fill:#ecfdf5,stroke:#10b981,stroke-width:3px,color:#065f46'
            : 'fill:#ecfdf5,stroke:#10b981,stroke-width:1.5px,color:#065f46',
          isRoot, trait?.description || '');

        if (depth < this.traceDepth) {
          // Reverse: agents implementing this trait
          this.agents
            .filter(a => (a.implements_traits || []).includes(traitName))
            .forEach(a => {
              const k = this.agentEntityKey(a.id);
              emitEdge(k, currentKey, 'implements');
              if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
            });
          // Reverse: skills implementing this trait
          this.skills
            .filter(s => (s.implements_traits || []).includes(traitName))
            .forEach(s => {
              const k = this.skillEntityKey(s.id!);
              emitEdge(k, currentKey, 'implements');
              if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
            });
        }
      } else if (currentKey.startsWith('mcp:')) {
        const mcpId = currentKey.slice('mcp:'.length);
        const mcp = this.mcpServers.find(m => m.id === mcpId);
        const toolsLabel = mcp?.tools_count ? ` (${mcp.tools_count} tools)` : '';
        const label = mcp
          ? `🔌 ${this.sanitizeLabel(mcp.server_name)}${isRoot ? toolsLabel : ''}`
          : `🔌 ${this.sanitizeLabel(mcpId)}`;
        emitNode(currentKey, label,
          isRoot
            ? 'fill:#fff7ed,stroke:#f97316,stroke-width:3px,color:#7c2d12'
            : 'fill:#fff7ed,stroke:#f97316,stroke-width:1.5px,color:#7c2d12',
          isRoot, mcp?.description || '');

        if (depth < this.traceDepth) {
          // Reverse: agents using this tool
          this.agents
            .filter(a => (a.attached_mcp_servers || []).includes(mcpId))
            .forEach(a => {
              const k = this.agentEntityKey(a.id);
              emitEdge(k, currentKey, 'uses MCP');
              if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
            });
          // Reverse: skills using this tool
          this.skills
            .filter(s => (s.attached_mcp_servers || []).includes(mcpId))
            .forEach(s => {
              const k = this.skillEntityKey(s.id!);
              emitEdge(k, currentKey, 'uses MCP');
              if (!queued.has(k)) { queued.add(k); queue.push([k, depth + 1]); }
            });
        }
      }
    }

    // Combine: nodes first, then edges
    lines.push(...nodeLines, ...edgeLines);

    if (lines.length === 1) {
      lines.push(`    noc["No connections found for this entity"]`);
    }

    return lines.join('\n');
  }

  async renderDiagram(code: string): Promise<void> {
    if (!this.mermaidContainer) return;
    this.renderError = null;
    this.diagramCounter++;
    const id = `mermaid-svg-${this.diagramCounter}`;
    try {
      const { svg } = await mermaid.render(id, code);
      this.mermaidContainer.nativeElement.innerHTML = svg;
    } catch (e: any) {
      console.error('Mermaid render error:', e);
      this.renderError = e?.message || 'Failed to render diagram.';
      if (this.mermaidContainer) {
        this.mermaidContainer.nativeElement.innerHTML = '';
      }
    }
  }

  onMermaidClick(event: MouseEvent): void {
    // Intercept clicks on links inside the mermaid diagram to use Angular router
    const target = event.target as HTMLElement;
    const anchor = target.closest('a');

    if (anchor && anchor.href) {
      const url = new URL(anchor.href);
      // Check if it's an internal link intended for routing
      if (url.origin === window.location.origin && url.pathname.startsWith('/network-visualizer/')) {
        event.preventDefault(); // Prevent full page reload
        this.router.navigateByUrl(url.pathname);
      }
    }
  }
}
