import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { of } from 'rxjs';
import {
  KnowledgeExplorerComponent,
  SystemTuple,
} from './knowledge-explorer.component';
import {
  ApiService,
  Agent,
  Skill,
  KnowledgeNode,
} from '../../services/api.service';

describe('KnowledgeExplorerComponent', () => {
  let component: KnowledgeExplorerComponent;
  let fixture: ComponentFixture<KnowledgeExplorerComponent>;
  let apiServiceSpy: jasmine.SpyObj<ApiService>;

  const mockAgentSummary: Agent = {
    id: 'agent-1',
    name: 'SecurityAgent',
    description: 'Security testing agent',
    tags: ['security'],
    implements_traits: [],
    uses_traits: [],
    current_version: '1.0.0',
    owner_id: '00000000-0000-0000-0000-000000000000',
    judge_threshold: 0.8,
  };

  const mockHydratedAgent: Agent = {
    ...mockAgentSummary,
    attached_skills: ['skill-1'],
    attached_tools: ['tool-1'],
    attached_agents: ['agent-2'],
    implements_traits: ['TraitAuditor'],
    uses_traits: ['TraitSandbox'],
  };

  const mockSkill: Skill = {
    id: 'skill-1',
    name: 'CodeAnalysisSkill',
    description: 'Analyzes code for vulnerabilities',
    definition: 'Instructions...',
    tags: ['analysis'],
    current_version: '1.0.0',
    owner_id: '00000000-0000-0000-0000-000000000000',
    attached_tools: ['tool-1'],
    attached_skills: [],
    implements_traits: ['TraitAuditor'],
  };

  const mockTools = [
    {
      id: 'tool-1',
      server_name: 'security-tools',
      name: 'VulnerabilityScanner',
      transport_type: 'HTTP',
      endpoint_config: { description: 'Scans for CVEs', tags: ['cve'] },
      cached_capabilities: { tools: [{ name: 'scan_cve' }] },
    },
  ];

  const mockTraits = {
    items: [
      {
        id: 'trait-1',
        name: 'TraitAuditor',
        description: 'Auditing trait contract',
        owner_id: '00000000-0000-0000-0000-000000000000',
        version: '1.0.0',
        capability_requirements: [],
        behavioral_invariants: [],
        evaluation_criteria: [],
        tags: [],
      },
      {
        id: 'trait-2',
        name: 'TraitSandbox',
        description: 'Sandbox trait contract',
        owner_id: '00000000-0000-0000-0000-000000000000',
        version: '1.0.0',
        capability_requirements: [],
        behavioral_invariants: [],
        evaluation_criteria: [],
        tags: [],
      },
    ],
  };

  const mockKnowledgeNodes: KnowledgeNode[] = [
    {
      id: 'kn-1',
      topic: 'SecurityGuidelines',
      title: 'Company Security Guidelines',
      description: 'Standards for secure development',
      content: 'Never store raw secrets',
      tags: ['security', 'compliance'],
    },
  ];

  const mockKnowledgeTuples = [
    {
      id: 'tup-1',
      source_node_id: 'kn-1',
      subject: 'Company Security Guidelines',
      predicate: 'governs',
      object: 'SecurityAgent',
      confidence: 0.95,
    },
  ];

  beforeEach(async () => {
    apiServiceSpy = jasmine.createSpyObj('ApiService', [
      'getAgents',
      'getAgent',
      'getSkills',
      'getTools',
      'getTraits',
      'getKnowledgeNodes',
      'getKnowledgeTuples',
    ]);

    apiServiceSpy.getAgents.and.returnValue(of([mockAgentSummary]));
    apiServiceSpy.getAgent.and.returnValue(of(mockHydratedAgent));
    apiServiceSpy.getSkills.and.returnValue(of([mockSkill]));
    apiServiceSpy.getTools.and.returnValue(of(mockTools));
    apiServiceSpy.getTraits.and.returnValue(of(mockTraits as any));
    apiServiceSpy.getKnowledgeNodes.and.returnValue(of(mockKnowledgeNodes));
    apiServiceSpy.getKnowledgeTuples.and.returnValue(of(mockKnowledgeTuples));

    await TestBed.configureTestingModule({
      imports: [KnowledgeExplorerComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([]),
        { provide: ApiService, useValue: apiServiceSpy },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(KnowledgeExplorerComponent);
    component = fixture.componentInstance;
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render the workspace title as an interactive view switcher trigger with dropdown affordance', () => {
    fixture.detectChanges();
    const switcher = fixture.nativeElement.querySelector(
      '[data-testid="workspace-title-switcher"]',
    );
    expect(switcher).toBeTruthy();
    expect(switcher.textContent).toContain('Knowledge Explorer');
    expect(switcher.textContent).toContain('expand_more');
  });

  it('should hydrate agents and extract relational tuples for has_skill, uses_tool, implements, and requires_trait', () => {
    component.loadData();

    // Verify hydration call
    expect(apiServiceSpy.getAgent).toHaveBeenCalledWith('agent-1');

    // Verify extracted tuples
    const tuples = component.allTuples;
    expect(tuples.length).toBeGreaterThanOrEqual(4);

    const hasSkillTuple = tuples.find((t) => t.predicate === 'has_skill');
    expect(hasSkillTuple).toBeTruthy();
    expect(hasSkillTuple?.subjectId).toBe('agent-1');
    expect(hasSkillTuple?.objectId).toBe('skill-1');

    const usesToolTuple = tuples.find(
      (t) => t.predicate === 'uses_tool' && t.subjectId === 'agent-1',
    );
    expect(usesToolTuple).toBeTruthy();
    expect(usesToolTuple?.objectId).toBe('tool-1');

    const skillUsesToolTuple = tuples.find(
      (t) => t.predicate === 'uses_tool' && t.subjectId === 'skill-1',
    );
    expect(skillUsesToolTuple).toBeTruthy();
    expect(skillUsesToolTuple?.objectId).toBe('tool-1');

    const implementsTuple = tuples.find(
      (t) => t.predicate === 'implements' && t.subjectId === 'agent-1',
    );
    expect(implementsTuple).toBeTruthy();
    expect(implementsTuple?.objectId).toBe('trait-1');

    const requiresTraitTuple = tuples.find(
      (t) => t.predicate === 'requires_trait',
    );
    expect(requiresTraitTuple).toBeTruthy();
    expect(requiresTraitTuple?.objectId).toBe('trait-2');
  });

  it('should configure vis-network edges with predicate labels, arrows, and tuple hover tooltips', () => {
    component.loadData();

    const edgeList = component.edges.get();
    expect(edgeList.length).toBeGreaterThan(0);

    const firstEdge: any = edgeList.find((e: any) => e.label === 'has_skill');
    expect(firstEdge).toBeTruthy();
    expect(firstEdge.arrows).toBe('to');
    expect(firstEdge.title).toContain('—[has_skill]→');
    expect(firstEdge.font).toBeDefined();
    expect(firstEdge.font.size).toBe(11);
  });

  it('should extract inbound and outbound tuples when an entity is selected', () => {
    component.loadData();

    // Select the agent
    component.onNodeSelected('agent-1');
    expect(component.selectedNode).toBeTruthy();
    expect(component.outboundTuples.length).toBeGreaterThan(0);

    const hasSkill = component.outboundTuples.find(
      (t) => t.predicate === 'has_skill',
    );
    expect(hasSkill).toBeTruthy();
    expect(hasSkill?.objectName).toBe('CodeAnalysisSkill');

    // Select the tool
    component.onNodeSelected('tool-1');
    expect(component.inboundTuples.length).toBeGreaterThanOrEqual(2); // Agent and Skill both use this tool
    const agentInbound = component.inboundTuples.find(
      (t) => t.subjectId === 'agent-1',
    );
    expect(agentInbound).toBeTruthy();
  });

  it('should filter edges by predicate when filterPredicate is set', () => {
    component.loadData();
    const totalEdges = component.edges.get().length;

    // Filter by uses_tool
    component.setPredicateFilter('uses_tool');
    const filteredEdges = component.edges.get();
    expect(filteredEdges.length).toBeLessThan(totalEdges);
    expect(
      filteredEdges.every((e: any) => e.predicate === 'uses_tool'),
    ).toBeTrue();

    // Reset to 'all'
    component.setPredicateFilter('all');
    expect(component.edges.get().length).toBe(totalEdges);
  });
});
