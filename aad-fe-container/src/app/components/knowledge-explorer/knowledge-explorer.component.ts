import {
  Component,
  OnInit,
  ElementRef,
  AfterViewInit,
  NgZone,
  ChangeDetectorRef,
} from '@angular/core';
import { CommonModule, TitleCasePipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { NavMenuItem, APP_NAV_MENU_ITEMS } from '../../models/navigation';
import {
  ApiService,
  Agent,
  Skill,
  KnowledgeNode,
} from '../../services/api.service';
import {
  ConceptGuideComponent,
  ConceptTabMapping,
} from '../concept-guide/concept-guide.component';
import { Network } from 'vis-network';
import { DataSet } from 'vis-data';
import { forkJoin, of } from 'rxjs';
import { catchError, map, switchMap } from 'rxjs/operators';

export interface SystemTuple {
  id: string;
  subjectId: string;
  subjectName: string;
  subjectType: 'agent' | 'skill' | 'tool' | 'trait' | 'knowledge';
  predicate: string;
  objectId: string;
  objectName: string;
  objectType: 'agent' | 'skill' | 'tool' | 'trait' | 'knowledge';
  confidence?: number;
}

@Component({
  selector: 'app-knowledge-explorer',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    RouterModule,
    MatIconModule,
    MatButtonModule,
    MatMenuModule,
    TitleCasePipe,
    ConceptGuideComponent,
  ],
  templateUrl: './knowledge-explorer.component.html',
  styleUrl: './knowledge-explorer.component.scss',
})
export class KnowledgeExplorerComponent implements OnInit, AfterViewInit {
  menuItems: NavMenuItem[] = APP_NAV_MENU_ITEMS;
  selectedNode: any = null;
  network: any;
  nodes: any = new DataSet([]);
  edges: any = new DataSet([]);

  // Physics state
  isPhysicsPanelOpen: boolean = false;
  physicsOptions = {
    gravitationalConstant: -28,
    centralGravity: 0.005,
    springLength: 220,
    springConstant: 0.16
  };

  // Tuple state
  allTuples: SystemTuple[] = [];
  allEdges: any[] = [];
  outboundTuples: SystemTuple[] = [];
  inboundTuples: SystemTuple[] = [];
  allEntities: any[] = [];

  // Filter state
  activePredicateFilter: string = 'all';
  availablePredicates: string[] = [];
  predicateCounts: { [key: string]: number } = {};

  // Zero-Footprint Concept Guide
  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'hub',
      iconColor: 'text-indigo-600',
      title: '1. Ecosystem Entity Nodes',
      description:
        'Interactive graph nodes representing Agents, Skills, Tools, Traits, and Knowledge documents.',
    },
    {
      icon: 'sync_alt',
      iconColor: 'text-amber-600',
      title: '2. Relational Tuples (SPO Triples)',
      description:
        'Edges represent directional Subject-Predicate-Object relations (e.g., uses_tool, has_skill, implements).',
    },
    {
      icon: 'account_tree',
      iconColor: 'text-emerald-600',
      title: '3. Multi-Hop Graph Traversal',
      description:
        'Explore multi-step dependencies and navigate directly from selected entities to connected dependencies.',
    },
  ];

  constructor(
    private el: ElementRef,
    private apiService: ApiService,
    private zone: NgZone,
    private cdr: ChangeDetectorRef,
  ) {}

  ngOnInit(): void {
    this.loadData();
  }

  ngAfterViewInit(): void {
    this.initGraph();
  }

  initGraph(): void {
    const container = this.el?.nativeElement?.querySelector('#network-graph');
    if (!container) {
      return;
    }

    const data = {
      nodes: this.nodes,
      edges: this.edges,
    };
    const options = {
      nodes: {
        shape: 'dot',
        size: 20,
        font: {
          size: 13,
          color: '#1e293b',
        },
        borderWidth: 2,
      },
      edges: {
        width: 1.5,
        color: { color: '#cbd5e1', highlight: '#6366f1' },
        smooth: {
          type: 'continuous',
        },
      },
      physics: {
        forceAtlas2Based: {
          gravitationalConstant: this.physicsOptions.gravitationalConstant,
          centralGravity: this.physicsOptions.centralGravity,
          springLength: this.physicsOptions.springLength,
          springConstant: this.physicsOptions.springConstant,
        },
        maxVelocity: 146,
        solver: 'forceAtlas2Based',
        timestep: 0.35,
        stabilization: { iterations: 150 },
      },
      interaction: {
        hover: true,
        tooltipDelay: 150,
        zoomView: true,
      },
    };

    this.network = new Network(container, data, options);

    this.network.on('selectNode', (params: any) => {
      this.zone.run(() => {
        const nodeId = params.nodes[0];
        this.onNodeSelected(nodeId);
      });
    });

    this.network.on('deselectNode', () => {
      this.zone.run(() => {
        this.selectedNode = null;
        this.outboundTuples = [];
        this.inboundTuples = [];
        this.cdr.markForCheck();
      });
    });
  }

  loadData(): void {
    forkJoin({
      agentsSummary: this.apiService.getAgents().pipe(catchError(() => of([]))),
      skills: this.apiService.getSkills().pipe(catchError(() => of([]))),
      tools: this.apiService.getTools().pipe(catchError(() => of([]))),
      traits: this.apiService
        .getTraits()
        .pipe(catchError(() => of({ items: [] } as any))),
      knowledge: this.apiService
        .getKnowledgeNodes()
        .pipe(catchError(() => of([]))),
    })
      .pipe(
        switchMap(({ agentsSummary, skills, tools, traits, knowledge }) => {
          // Hydrate each agent in parallel
          const agentList = (agentsSummary as Agent[]) || [];
          const agentHydrations = agentList.map((a) =>
            this.apiService.getAgent(a.id).pipe(catchError(() => of(a))),
          );
          const agentsObs =
            agentHydrations.length > 0
              ? forkJoin(agentHydrations)
              : of([] as Agent[]);

          // Fetch tuples for knowledge nodes in parallel
          const knList = (knowledge as KnowledgeNode[]) || [];
          const knTupleObs = knList.map((kn) =>
            this.apiService
              .getKnowledgeTuples(kn.id)
              .pipe(catchError(() => of([]))),
          );
          const tuplesObs =
            knTupleObs.length > 0 ? forkJoin(knTupleObs) : of([]);

          // Hydrate traits
          let directTraits: any[] = [];
          if (Array.isArray((traits as any)?.items)) {
            directTraits = (traits as any).items;
          } else if (Array.isArray(traits)) {
            directTraits = traits;
          }

          const traitIds: string[] = Array.isArray((traits as any)?.ids)
            ? (traits as any).ids
            : [];

          const traitHydrations = traitIds.map((tid) =>
            this.apiService.getTrait(tid).pipe(catchError(() => of(null))),
          );
          const traitsObs =
            traitHydrations.length > 0
              ? forkJoin(traitHydrations).pipe(
                  map((fetched) => [
                    ...directTraits,
                    ...fetched.filter(Boolean),
                  ]),
                )
              : of(directTraits);

          return forkJoin({
            agents: agentsObs,
            skills: of((skills as Skill[]) || []),
            tools: of((tools as any[]) || []),
            traits: traitsObs,
            knowledge: of(knList),
            knowledgeTuplesList: tuplesObs,
          });
        }),
      )
      .subscribe((results) => {
        const newNodes: any[] = [];
        const extractedTuples: SystemTuple[] = [];

        // Dictionaries for entity resolution by ID and by lowercase Name
        const entityById = new Map<
          string,
          { id: string; name: string; type: any; raw: any }
        >();
        const entityByName = new Map<
          string,
          { id: string; name: string; type: any; raw: any }
        >();

        const registerEntity = (
          id: string,
          name: string,
          type: any,
          raw: any,
        ) => {
          const entry = { id, name, type, raw };
          entityById.set(id, entry);
          if (name) {
            entityByName.set(name.toLowerCase(), entry);
          }
        };

        // 1. Process Agents
        results.agents.forEach((agent) => {
          registerEntity(agent.id, agent.name, 'agent', agent);
          newNodes.push({
            id: agent.id,
            label: agent.name,
            color: {
              background: '#f3e8ff',
              border: '#a855f7',
              highlight: { background: '#e9d5ff', border: '#9333ea' },
            },
            rawData: { ...agent, type: 'agent' },
          });
        });

        // 2. Process Skills
        results.skills.forEach((skill) => {
          const skillId = skill.id || skill.name;
          registerEntity(skillId, skill.name, 'skill', skill);
          newNodes.push({
            id: skillId,
            label: skill.name,
            color: {
              background: '#fef3c7',
              border: '#f59e0b',
              highlight: { background: '#fde68a', border: '#d97706' },
            },
            rawData: { ...skill, id: skillId, type: 'skill' },
          });
        });

        // 3. Process Tools
        results.tools.forEach((tool: any) => {
          const toolId = tool.id || tool.server_name;
          const toolName = tool.name || tool.server_name;
          registerEntity(toolId, toolName, 'tool', tool);
          if (tool.server_name) {
            entityByName.set(tool.server_name.toLowerCase(), {
              id: toolId,
              name: toolName,
              type: 'tool',
              raw: tool,
            });
          }
          newNodes.push({
            id: toolId,
            label: toolName,
            color: {
              background: '#dbeafe',
              border: '#3b82f6',
              highlight: { background: '#bfdbfe', border: '#2563eb' },
            },
            rawData: { ...tool, id: toolId, name: toolName, type: 'tool' },
          });
        });

        // 4. Process Traits
        const traitsList: any[] = Array.isArray(results.traits)
          ? results.traits
          : Array.isArray((results.traits as any)?.items)
            ? (results.traits as any).items
            : [];

        traitsList.forEach((trait) => {
          const traitId = trait.id || trait.name;
          registerEntity(traitId, trait.name, 'trait', trait);
          newNodes.push({
            id: traitId,
            label: trait.name,
            color: {
              background: '#d1fae5',
              border: '#10b981',
              highlight: { background: '#a7f3d0', border: '#059669' },
            },
            rawData: { ...trait, id: traitId, type: 'trait' },
          });
        });

        // 5. Process Knowledge Nodes
        results.knowledge.forEach((node) => {
          const title = node.title || node.topic;
          registerEntity(node.id, title, 'knowledge', node);
          newNodes.push({
            id: node.id,
            label: title,
            shape: 'square',
            color: {
              background: '#e0e7ff',
              border: '#6366f1',
              highlight: { background: '#c7d2fe', border: '#4f46e5' },
            },
            rawData: { ...node, name: title, type: 'knowledge' },
          });
        });

        // Helper to resolve entity by ID or name
        const resolveEntity = (ref: string) => {
          if (!ref) return undefined;
          return entityById.get(ref) || entityByName.get(ref.toLowerCase());
        };

        // Helper to add a tuple
        let tupleCounter = 1;
        const addTuple = (
          subject: { id: string; name: string; type: any },
          predicate: string,
          object: { id: string; name: string; type: any },
          confidence?: number,
        ) => {
          // Prevent duplicate tuples
          const exists = extractedTuples.some(
            (t) =>
              t.subjectId === subject.id &&
              t.predicate === predicate &&
              t.objectId === object.id,
          );
          if (!exists) {
            extractedTuples.push({
              id: `tuple-${tupleCounter++}`,
              subjectId: subject.id,
              subjectName: subject.name,
              subjectType: subject.type,
              predicate,
              objectId: object.id,
              objectName: object.name,
              objectType: object.type,
              confidence,
            });
          }
        };

        // Extract tuples from Agents
        results.agents.forEach((agent) => {
          const subject = {
            id: agent.id,
            name: agent.name,
            type: 'agent' as const,
          };

          // Agent -> Skill (has_skill)
          if (agent.attached_skills) {
            agent.attached_skills.forEach((skillRef) => {
              const target = resolveEntity(skillRef);
              if (target) {
                addTuple(subject, 'has_skill', target);
              }
            });
          }

          // Agent -> Tool (uses_tool)
          if (agent.attached_tools) {
            agent.attached_tools.forEach((toolRef) => {
              const target = resolveEntity(toolRef);
              if (target) {
                addTuple(subject, 'uses_tool', target);
              }
            });
          }

          // Agent -> Agent (delegates_to)
          if (agent.attached_agents) {
            agent.attached_agents.forEach((agentRef) => {
              const target = resolveEntity(agentRef);
              if (target && target.id !== agent.id) {
                addTuple(subject, 'delegates_to', target);
              }
            });
          }

          // Agent -> Trait (implements)
          if (agent.implements_traits) {
            agent.implements_traits.forEach((traitRef) => {
              let target = resolveEntity(traitRef);
              if (!target && traitRef) {
                const tid = `trait-${traitRef.toLowerCase().replace(/\s+/g, '-')}`;
                registerEntity(tid, traitRef, 'trait', {
                  id: tid,
                  name: traitRef,
                  type: 'trait',
                });
                newNodes.push({
                  id: tid,
                  label: traitRef,
                  color: {
                    background: '#d1fae5',
                    border: '#10b981',
                    highlight: { background: '#a7f3d0', border: '#059669' },
                  },
                  rawData: { id: tid, name: traitRef, type: 'trait' },
                });
                target = resolveEntity(tid);
              }
              if (target) {
                addTuple(subject, 'implements', target);
              }
            });
          }

          // Agent -> Trait (requires_trait)
          if (agent.uses_traits) {
            agent.uses_traits.forEach((traitRef) => {
              let target = resolveEntity(traitRef);
              if (!target && traitRef) {
                const tid = `trait-${traitRef.toLowerCase().replace(/\s+/g, '-')}`;
                registerEntity(tid, traitRef, 'trait', {
                  id: tid,
                  name: traitRef,
                  type: 'trait',
                });
                newNodes.push({
                  id: tid,
                  label: traitRef,
                  color: {
                    background: '#d1fae5',
                    border: '#10b981',
                    highlight: { background: '#a7f3d0', border: '#059669' },
                  },
                  rawData: { id: tid, name: traitRef, type: 'trait' },
                });
                target = resolveEntity(tid);
              }
              if (target) {
                addTuple(subject, 'requires_trait', target);
              }
            });
          }
        });

        // Extract tuples from Skills
        results.skills.forEach((skill) => {
          const skillId = skill.id || skill.name;
          const subject = {
            id: skillId,
            name: skill.name,
            type: 'skill' as const,
          };

          // Skill -> Tool (uses_tool)
          if (skill.attached_tools) {
            skill.attached_tools.forEach((toolRef) => {
              const target = resolveEntity(toolRef);
              if (target) {
                addTuple(subject, 'uses_tool', target);
              }
            });
          }

          // Skill -> Skill (composes_skill)
          if (skill.attached_skills) {
            skill.attached_skills.forEach((subSkillRef) => {
              const target = resolveEntity(subSkillRef);
              if (target && target.id !== skillId) {
                addTuple(subject, 'composes_skill', target);
              }
            });
          }

          // Skill -> Trait (implements)
          if (skill.implements_traits) {
            skill.implements_traits.forEach((traitRef) => {
              let target = resolveEntity(traitRef);
              if (!target && traitRef) {
                const tid = `trait-${traitRef.toLowerCase().replace(/\s+/g, '-')}`;
                registerEntity(tid, traitRef, 'trait', {
                  id: tid,
                  name: traitRef,
                  type: 'trait',
                });
                newNodes.push({
                  id: tid,
                  label: traitRef,
                  color: {
                    background: '#d1fae5',
                    border: '#10b981',
                    highlight: { background: '#a7f3d0', border: '#059669' },
                  },
                  rawData: { id: tid, name: traitRef, type: 'trait' },
                });
                target = resolveEntity(tid);
              }
              if (target) {
                addTuple(subject, 'implements', target);
              }
            });
          }
        });

        // Extract factual tuples from Knowledge Nodes
        if (Array.isArray(results.knowledgeTuplesList)) {
          results.knowledgeTuplesList.forEach((tuplesForNode: any[]) => {
            if (Array.isArray(tuplesForNode)) {
              tuplesForNode.forEach((kt) => {
                const sourceNode =
                  resolveEntity(kt.source_node_id) || resolveEntity(kt.subject);
                let targetNode = resolveEntity(kt.object);
                if (!targetNode && kt.object) {
                  const conceptId = `concept-${kt.object.toLowerCase().replace(/\s+/g, '-')}`;
                  registerEntity(conceptId, kt.object, 'knowledge', {
                    id: conceptId,
                    name: kt.object,
                    type: 'knowledge',
                  });
                  newNodes.push({
                    id: conceptId,
                    label: kt.object,
                    shape: 'box',
                    color: {
                      background: '#f8fafc',
                      border: '#94a3b8',
                      highlight: { background: '#e2e8f0', border: '#64748b' },
                    },
                    rawData: {
                      id: conceptId,
                      name: kt.object,
                      title: kt.object,
                      type: 'knowledge',
                    },
                  });
                  targetNode = resolveEntity(conceptId);
                }
                if (
                  sourceNode &&
                  targetNode &&
                  sourceNode.id !== targetNode.id
                ) {
                  addTuple(
                    sourceNode,
                    kt.predicate || 'relates_to',
                    targetNode,
                    kt.confidence,
                  );
                }
              });
            }
          });
        }

        this.allTuples = extractedTuples;
        this.allEntities = Array.from(entityById.values());

        // Build vis-network edges from tuples
        this.allEdges = this.allTuples.map((tuple) => ({
          id: tuple.id,
          from: tuple.subjectId,
          to: tuple.objectId,
          label: tuple.predicate,
          arrows: 'to',
          font: {
            size: 11,
            color: '#475569',
            background: '#ffffff',
            strokeWidth: 2,
            strokeColor: '#f8fafc',
            align: 'horizontal',
          },
          color: {
            color: this.getEdgeColor(tuple.predicate),
            highlight: '#6366f1',
          },
          title: `(${tuple.subjectName}) —[${tuple.predicate}]→ (${tuple.objectName})`,
          smooth: { type: 'continuous' },
          predicate: tuple.predicate,
        }));

        // Calculate predicate counts
        const counts: { [key: string]: number } = { all: this.allEdges.length };
        this.allEdges.forEach((e) => {
          counts[e.predicate] = (counts[e.predicate] || 0) + 1;
        });
        this.predicateCounts = counts;
        this.availablePredicates = [
          'all',
          ...Object.keys(counts).filter((k) => k !== 'all'),
        ];

        // Update datasets
        this.nodes.clear();
        this.edges.clear();
        this.nodes.add(newNodes);
        this.edges.add(this.allEdges);

        if (this.network) {
          this.network.fit();
        }

        // If a node was previously selected, re-evaluate its tuples
        if (this.selectedNode) {
          this.onNodeSelected(this.selectedNode.id);
        }
      });
  }

  onNodeSelected(nodeId: string): void {
    const node: any = this.nodes.get(nodeId);
    if (node) {
      this.selectedNode = node.rawData;
    } else {
      const entity = this.allEntities.find((e) => e.id === nodeId);
      this.selectedNode = entity
        ? entity.raw
        : { id: nodeId, name: nodeId, type: 'unknown' };
    }

    this.outboundTuples = this.allTuples.filter((t) => t.subjectId === nodeId);
    this.inboundTuples = this.allTuples.filter((t) => t.objectId === nodeId);
    this.cdr.markForCheck();
  }

  selectAndFocusNode(nodeId: string): void {
    this.zone.run(() => {
      if (this.network) {
        try {
          this.network.focus(nodeId, {
            scale: 1.2,
            animation: { duration: 400, easingFunction: 'easeInOutQuad' },
          });
          this.network.selectNodes([nodeId]);
        } catch (e) {
          // Safe fallback
        }
      }
      this.onNodeSelected(nodeId);
    });
  }

  setPredicateFilter(predicate: string): void {
    this.activePredicateFilter = predicate;
    this.edges.clear();
    if (predicate === 'all') {
      this.edges.add(this.allEdges);
    } else {
      this.edges.add(this.allEdges.filter((e) => e.predicate === predicate));
    }
  }

  closePanel(): void {
    this.selectedNode = null;
    this.outboundTuples = [];
    this.inboundTuples = [];
    if (this.network) {
      this.network.unselectAll();
    }
  }

  getEdgeColor(predicate: string): string {
    switch (predicate) {
      case 'uses_tool':
        return '#3b82f6'; // Blue
      case 'has_skill':
        return '#f59e0b'; // Amber
      case 'delegates_to':
        return '#a855f7'; // Purple
      case 'implements':
        return '#10b981'; // Emerald
      case 'requires_trait':
        return '#0d9488'; // Teal
      case 'composes_skill':
        return '#ea580c'; // Orange
      default:
        return '#6366f1'; // Indigo
    }
  }

  getPredicateBadgeColor(predicate: string): string {
    switch (predicate) {
      case 'uses_tool':
        return 'bg-blue-50 text-blue-700 border-blue-200';
      case 'has_skill':
        return 'bg-amber-50 text-amber-700 border-amber-200';
      case 'delegates_to':
        return 'bg-purple-50 text-purple-700 border-purple-200';
      case 'implements':
        return 'bg-emerald-50 text-emerald-700 border-emerald-200';
      case 'requires_trait':
        return 'bg-teal-50 text-teal-700 border-teal-200';
      case 'composes_skill':
        return 'bg-orange-50 text-orange-700 border-orange-200';
      default:
        return 'bg-indigo-50 text-indigo-700 border-indigo-200';
    }
  }

  getIcon(type: string): string {
    switch (type) {
      case 'agent':
        return 'smart_toy';
      case 'skill':
        return 'extension';
      case 'tool':
        return 'dns';
      case 'trait':
        return 'verified';
      case 'knowledge':
        return 'library_books';
      default:
        return 'help_outline';
    }
  }

  getIconColor(type: string): string {
    switch (type) {
      case 'agent':
        return 'text-purple-600';
      case 'skill':
        return 'text-amber-600';
      case 'tool':
        return 'text-blue-600';
      case 'trait':
        return 'text-emerald-600';
      case 'knowledge':
        return 'text-indigo-600';
      default:
        return 'text-slate-600';
    }
  }

  updatePhysics(): void {
    if (this.network) {
      this.network.setOptions({
        physics: {
          forceAtlas2Based: {
            gravitationalConstant: this.physicsOptions.gravitationalConstant,
            centralGravity: this.physicsOptions.centralGravity,
            springLength: this.physicsOptions.springLength,
            springConstant: this.physicsOptions.springConstant
          }
        }
      });
    }
  }
}
