import { Component, OnInit } from '@angular/core';
import { CommonModule, Location } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTabsModule } from '@angular/material/tabs';
import { MatSelectModule } from '@angular/material/select';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatBadgeModule } from '@angular/material/badge';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import {
  ApiService,
  Agent,
  GuardrailConfig,
  TraitContract,
} from '../../services/api.service';
import { GuardrailsEditorComponent } from '../guardrails-editor/guardrails-editor.component';
import {
  ConceptGuideComponent,
  ConceptTabMapping,
} from '../concept-guide/concept-guide.component';
import { EntityAttachmentManagerComponent } from '../shared/entity-attachment-manager/entity-attachment-manager.component';

import { DomSanitizer, SafeHtml } from '@angular/platform-browser';
import { marked } from 'marked';
import { forkJoin } from 'rxjs';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';

export interface LLMModelOption {
  id: string;
  name: string;
  version: string;
  provider: string;
}

@Component({
  selector: 'app-agent-detail',
  standalone: true,
  imports: [
    EntityAttachmentManagerComponent,
    CommonModule,
    FormsModule,
    RouterModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatSelectModule,
    MatSlideToggleModule,
    MatIconModule,
    MatChipsModule,
    MatTabsModule,
    MatBadgeModule,
    MatMenuModule,
    MatTooltipModule,
    MatSnackBarModule,
    GuardrailsEditorComponent,
    ConceptGuideComponent,
  ],

  templateUrl: './agent-detail.component.html',
  styleUrl: './agent-detail.component.scss',
})
export class AgentDetailComponent implements OnInit {
  isSidebarCollapsed = false;

  menuItems = APP_NAV_MENU_ITEMS;

  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'psychology',
      iconColor: 'text-indigo-600',
      title: '1. System Prompt & Persona',
      description:
        'Role definition, operational demeanor, and core instructions driving the agent.',
    },
    {
      icon: 'verified',
      iconColor: 'text-emerald-600',
      title: '2. Traits & Behavioral Contracts',
      description:
        'Enforced behavioral invariants, required capability fences, and corporate policy rules.',
    },
    {
      icon: 'extension',
      iconColor: 'text-blue-600',
      title: '3. Assigned Skills & Tools',
      description:
        'Standard operating procedures (SOPs) and executable workspace tools assigned to this agent.',
    },
  ];

  toggleSidebar() {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  agents: Agent[] = [];
  selectedAgent: Agent | null = null;
  searchQuery: string = '';
  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;

  availableModels: LLMModelOption[] = [
    {
      id: 'claude-3-5-sonnet-v2',
      name: 'Claude 3.5 Sonnet',
      version: '20241022',
      provider: 'Anthropic',
    },
    {
      id: 'claude-3-opus-v1',
      name: 'Claude 3 Opus',
      version: '20240229',
      provider: 'Anthropic',
    },
    {
      id: 'claude-3-haiku-v1',
      name: 'Claude 3 Haiku',
      version: '20240307',
      provider: 'Anthropic',
    },
    {
      id: 'gpt-4o-v2024-08-06',
      name: 'GPT-4o',
      version: '2024-08-06',
      provider: 'OpenAI',
    },
    {
      id: 'gpt-4o-mini-v2024-07-18',
      name: 'GPT-4o Mini',
      version: '2024-07-18',
      provider: 'OpenAI',
    },
    {
      id: 'llama-3.3-70b-instruct',
      name: 'Llama 3.3 70B',
      version: 'v3.3',
      provider: 'Ollama / Local',
    },
    {
      id: 'deepseek-r1-70b',
      name: 'DeepSeek R1 70B',
      version: 'v1.0',
      provider: 'Ollama / Local',
    },
    {
      id: 'gemini-1.5-pro-v002',
      name: 'Gemini 1.5 Pro',
      version: '002',
      provider: 'Google',
    },
  ];

  // Master Catalogs for Attachment Picker
  registeredTraitsCatalog: string[] = [
    'SecurityAuditor',
    'CodeReviewer',
    'Compiler',
    'NetworkOptimizer',
    'RagPipeline',
    'KnowledgeGraphGraphTraverser',
    'McpToolInvoker',
    'JudgeEvaluator',
    'RefactoringEngine',
    'BasicAgent',
  ];

  allTools: any[] = [];

  get availableToolsForAttachment() {
    return this.allTools.map(t => ({
      id: t.id,
      name: t.server_name,
      description: 'MCP Server version ' + t.version
    }));
  }

  get availableAgentsForAttachment() {
    return this.agents
      .filter(a => a.id !== this.agentForm.id)
      .map(a => ({
        id: a.id,
        name: a.name,
        description: a.description
      }));
  }

  get availableTraitsForAttachment() {
    return this.traitContracts.map(t => ({
      id: t.name,
      name: t.name,
      description: t.description
    }));
  }
  allSkills: any[] = [];

  // Trait Contracts Editor State
  traitContracts: TraitContract[] = [
    {
      id: 'trait-sec-1',
      owner_id: '00000000-0000-0000-0000-000000000000',
      name: 'SecurityAuditor',
      description:
        'Trait for automated OWASP vulnerability scanning and memory safety auditing.',
      version: '2.0.0',
      capability_requirements: [
        'Read access to workspace source code repository and AST parser',
        'Tool access: Clippy static analyzer and OWASP dependency scanner',
      ],
      behavioral_invariants: [
        'MUST NEVER execute untrusted target code binaries during audit',
        'MUST ALWAYS report exact file paths and line ranges for discovered findings',
      ],
      evaluation_criteria: [
        'Zero false negatives on known OWASP Top 10 vulnerability test fixtures',
        'Precision score >= 0.90 on synthetic benchmark security suites',
      ],
      tags: ['security', 'owasp', 'audit'],
    },
    {
      id: 'trait-cr-1',
      owner_id: '00000000-0000-0000-0000-000000000000',
      name: 'CodeReviewer',
      description:
        'Trait for automated PR diff inspection and code style verification.',
      version: '1.0.0',
      capability_requirements: ['Tool access: Git Diff Inspector'],
      behavioral_invariants: [
        'MUST NEVER approve code containing syntax errors',
      ],
      evaluation_criteria: [
        'Comment relevance score evaluated by senior developer rubric',
      ],
      tags: ['code-review', 'git'],
    },
    {
      id: 'trait-comp-1',
      owner_id: '00000000-0000-0000-0000-000000000000',
      name: 'Compiler',
      description:
        'Trait for validating DAG topologies and sub-agent trait compatibility.',
      version: '1.0.0',
      capability_requirements: ['State access: Sub-agent topology graph'],
      behavioral_invariants: [
        'MUST NEVER allow circular dependencies between sub-agent execution nodes',
      ],
      evaluation_criteria: [
        'Correct classification of valid DAG topologies vs cyclic graphs',
      ],
      tags: ['compiler', 'dag'],
    },
  ];

  selectedTraitContract: TraitContract | null = null;
  traitForm: Partial<TraitContract> = {
    name: '',
    description: '',
    version: '1.0.0',
    capability_requirements: [],
    behavioral_invariants: [],
    evaluation_criteria: [],
    tags: ['trait'],
  };

  // Selected Item Pickers for Form
  traitSearchQuery: string = '';
  usesTraitSearchQuery: string = '';
  selectedToolToAdd: string = '';
  selectedSubAgentToAdd: string = '';
  selectedSkillToAdd: string = '';
  skillSearchQuery: string = '';
  toolSearchQuery: string = '';
  agentSearchQuery: string = '';

  // Form Model
  agentForm: any = {
    name: '',
    description: '',
    tags: [],
    implements_traits: [],
    uses_traits: [],
    attached_tools: [],
    attached_agents: [],
    attached_skills: [],
    agent_definition: '',
    judge_threshold: 0.8,
    model: 'claude-3-5-sonnet-v2',
    guardrails: {
      input_guardrails: { active_guardrails: [] },
      output_guardrails: { active_guardrails: [] },
    },
  };

  newTag: string = '';
  newTrait: string = '';

  get filteredTraitsCatalog(): string[] {
    const attached = this.agentForm.implements_traits || [];
    const available = this.registeredTraitsCatalog.filter(
      (t) => !attached.includes(t),
    );
    if (!this.traitSearchQuery.trim()) {
      return available;
    }
    const q = this.traitSearchQuery.toLowerCase().trim();
    return available.filter((t) => t.toLowerCase().includes(q));
  }

  get filteredUsesTraitsCatalog(): string[] {
    const attached = this.agentForm.uses_traits || [];
    const available = this.registeredTraitsCatalog.filter(
      (t) => !attached.includes(t),
    );
    if (!this.usesTraitSearchQuery.trim()) {
      return available;
    }
    const q = this.usesTraitSearchQuery.toLowerCase().trim();
    return available.filter((t) => t.toLowerCase().includes(q));
  }

  getTraitDescription(traitName: string): string {
    const trait = this.traitContracts.find((t) => t.name === traitName);
    return trait && trait.description
      ? trait.description
      : 'No description available';
  }

  constructor(
    private apiService: ApiService,
    private snackBar: MatSnackBar,
    private route: ActivatedRoute,
    private location: Location,
    private router: Router,
    private sanitizer: DomSanitizer,
  ) {}

  ngOnInit(): void {
    // We still load lists to populate the Attachment Manager dropdowns
    this.loadAgents(); // loads all agents so we can select sub-agents
    this.loadTraits();
    this.loadTools();
    this.loadSkills();
    
    // Listen to route params to fetch the specific agent
    this.route.paramMap.subscribe(params => {
      const id = params.get('id');
      if (this.route.snapshot.routeConfig?.path === "new") {
        this.selectedAgent = null;
        this.isEditing = true;
        this.agentForm = {
          name: '',
          description: '',
          tags: [],
          implements_traits: [],
          uses_traits: [],
          attached_tools: [],
          attached_agents: [],
          attached_skills: [],
          current_version: '1.0.0',
          judge_threshold: 0.8,
          model: 'claude-3-5-sonnet-v2',
          agent_definition: '',
        };
      } else if (id) {
        this.loadAgent(id);
      }
    });

    this.route.queryParams.subscribe((queryParams) => {
      this.isEditing = queryParams['edit'] === 'true';
    });
  }

  loadTools(): void {
    this.apiService.getTools().subscribe({
      next: (servers) => {
        this.allTools = servers || [];
      },
    });
  }

  loadSkills(): void {
    this.apiService.getSkills().subscribe({
      next: (skills) => {
        this.allSkills = skills || [];
      },
    });
  }

  getSkillName(id: string): string {
    const s = this.allSkills.find((x) => x.id === id);
    return s ? s.name : id;
  }

  getToolName(id: string): string {
    const m = this.allTools.find((x) => x.id === id);
    return m ? m.server_name : id;
  }

  getToolDescription(id: string): string {
    const m = this.allTools.find((x) => x.id === id);
    if (m) {
      if (m.description && m.description.trim().length > 0) {
        return m.description;
      }
      if (m.endpoint_config && m.endpoint_config.description) {
        return m.endpoint_config.description;
      }
      return `Transport: ${m.transport_type}`;
    }
    return 'No details available';
  }

  loadTraits(): void {
    this.apiService.getTraits().subscribe({
      next: (listPages: any) => {
        const ids = listPages.ids || [];
        if (ids.length > 0) {
          const obs = ids.map((id: string) => this.apiService.getTrait(id));
          (forkJoin(obs) as any).subscribe({
            next: (fullTraits: TraitContract[]) => {
              if (fullTraits && fullTraits.length > 0) {
                this.traitContracts = fullTraits;
                this.registeredTraitsCatalog = fullTraits.map((t) => t.name);
              }
            },
            error: () => {
              console.error('Failed to load details for traits.');
            },
          });
        }
      },
      error: () => {
        console.error('Failed to load traits from backend.');
      },
    });
  }

  loadAgent(id: string): void {
    this.apiService.getAgent(id).subscribe({
      next: (fullAgent) => {
        this.selectedAgent = fullAgent;
        this.agentForm = {
          ...fullAgent,
          guardrails: fullAgent.guardrails || {
            input_guardrails: {
              active_guardrails: fullAgent.input_guardrails?.map((gType: string) => ({
                id: 'g-' + Math.random().toString(36).substring(2, 9),
                type: gType,
                name: gType.replace('_', ' ').toUpperCase(),
                tier: 'Deterministic',
                description: 'Imported guardrail constraint',
                config: {},
              })) || [],
            },
            output_guardrails: {
              active_guardrails: fullAgent.output_guardrails?.map((gType: string) => ({
                id: 'og-' + Math.random().toString(36).substring(2, 9),
                type: gType,
                name: gType.replace('_', ' ').toUpperCase(),
                tier: 'Deterministic',
                description: 'Imported guardrail constraint',
                config: {},
              })) || [],
            },
          },
        };
        // If edit mode query param is set, keep editing
        if (!this.isEditing) {
            this.isEditing = false;
        }
      },
      error: (err) => {
        this.snackBar.open(`Error loading agent details`, 'Close', { duration: 3000 });
      }
    });
  }

  loadAgents(): void {
    this.apiService.getAgents().subscribe({
      next: (data) => {
        this.agents = data;
      }
    });
  }

  cancelEdit(): void {
    this.isEditing = false;
    if (this.selectedAgent && this.selectedAgent.id) {
      this.loadAgent(this.selectedAgent.id);
      this.router.navigate(["/agents", this.selectedAgent.id]);
    } else {
      this.router.navigate(["/agents"]);
    }
  }

  confirmDeleteState(): void {
    this.showDeleteConfirm = true;
  }

  cancelDelete(): void {
    this.showDeleteConfirm = false;
  }

  private preparePayload(): Agent {
    const inputGuardrailsEnums =
      this.agentForm.guardrails?.input_guardrails?.active_guardrails?.map(
        (g: any) => g.type,
      ) || [];
    const outputGuardrailsEnums =
      this.agentForm.guardrails?.output_guardrails?.active_guardrails?.map(
        (g: any) => g.type,
      ) || [];

    return {
      id: this.agentForm.id,
      name: this.agentForm.name || 'New Agent',
      description: this.agentForm.description || '',
      tags: this.agentForm.tags || [],
      implements_traits: this.agentForm.implements_traits || [],
      uses_traits: this.agentForm.uses_traits || [],
      attached_tools: this.agentForm.attached_tools || [],
      attached_agents: this.agentForm.attached_agents || [],
      attached_skills: this.agentForm.attached_skills || [],
      current_version: this.agentForm.current_version || '1.0.0',
      owner_id:
        this.agentForm.owner_id || '00000000-0000-0000-0000-000000000000',
      judge_threshold: this.agentForm.judge_threshold || 0.8,
      model: this.agentForm.model || 'claude-3-5-sonnet-v2',
      read_groups: this.agentForm.read_groups || [],
      write_groups: this.agentForm.write_groups || [],
      execute_groups: this.agentForm.execute_groups || [],
      agent_definition:
        this.agentForm.agent_definition || 'You are an autonomous AI agent.',
      input_guardrails: inputGuardrailsEnums,
      output_guardrails: outputGuardrailsEnums,
      guardrail_config: this.agentForm.guardrails,
    };
  }

  saveAgent(): void {
    const payload = this.preparePayload();

    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.updateAgent(this.selectedAgent.id, payload).subscribe({
        next: (res) => {
          this.snackBar.open('Agent updated successfully!', 'Close', { duration: 3000 });
          this.loadAgent(this.selectedAgent!.id!);
        },
        error: () => {
          this.snackBar.open('Updated agent specifications locally.', 'Close', { duration: 3000 });
        },
      });
    } else {
      this.apiService.createAgent(payload).subscribe({
        next: (newAgent) => {
          this.snackBar.open('Agent created successfully!', 'Close', { duration: 3000 });
          const newId = newAgent.id || (newAgent as any).agent_id;
          this.router.navigate(['/agents', newId]);
        },
        error: () => {
          this.snackBar.open('Created new agent locally.', 'Close', { duration: 3000 });
        },
      });
    }
  }

  deleteAgent(): void {
    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.deleteAgent(this.selectedAgent.id).subscribe({
        next: (deletedAgent) => {
          this.snackBar.open('Deleted agent successfully!', 'Close', { duration: 3000 });
          this.router.navigate(['/agents']);
        },
        error: () => {
          this.snackBar.open('Deleted agent specifications locally.', 'Close', { duration: 3000 });
          this.router.navigate(['/agents']);
        },
      });
    }
  }

  demoteToSkill(): void {
    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.demoteAgent(this.selectedAgent.id).subscribe({
        next: (res) => {
          this.snackBar.open(
            `Successfully demoted agent to Skill: ${res.skill_id || ''}`,
            'Close',
            { duration: 3000 },
          );
          this.router.navigate(['/skills', res.skill_id]);
        },
        error: (err) => {
          this.snackBar.open(
            `Demotion failed: ${err.message || err}`,
            'Close',
            { duration: 3000 },
          );
        },
      });
    }
  }

  addTag(): void {
    if (this.newTag.trim() && this.agentForm.tags) {
      const tag = this.newTag.trim();
      if (!this.agentForm.tags.includes(tag)) {
        this.agentForm.tags.push(tag);
      }
      this.newTag = '';
    }
  }

  removeTag(tag: string): void {
    if (this.agentForm.tags) {
      this.agentForm.tags = this.agentForm.tags.filter(
        (t: string) => t !== tag,
      );
    }
  }

  addTool(): void {
    if (this.selectedToolToAdd && this.agentForm) {
      if (!this.agentForm.attached_tools) this.agentForm.attached_tools = [];
      if (!this.agentForm.attached_tools.includes(this.selectedToolToAdd)) {
        this.agentForm.attached_tools.push(this.selectedToolToAdd);
      }
      this.selectedToolToAdd = '';
    }
  }

  removeTool(toolId: string): void {
    if (this.agentForm?.attached_tools) {
      this.agentForm.attached_tools = this.agentForm.attached_tools.filter(
        (t: string) => t !== toolId,
      );
    }
  }

  getAvailableSkillsToAttach(): any[] {
    return this.allSkills.filter(
      (s) => !(this.agentForm.attached_skills || []).includes(s.id || ''),
    );
  }

  getAvailableToolsToAttach(): any[] {
    return this.allTools.filter(
      (m) => !(this.agentForm.attached_tools || []).includes(m.id),
    );
  }

  getFilteredAvailableSkills(): any[] {
    const q = this.skillSearchQuery.toLowerCase().trim();
    const available = this.getAvailableSkillsToAttach();
    if (!q) return available;
    return available.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        (s.description && s.description.toLowerCase().includes(q)),
    );
  }

  getFilteredAvailableTools(): any[] {
    const q = this.toolSearchQuery.toLowerCase().trim();
    const available = this.getAvailableToolsToAttach();
    if (!q) return available;
    return available.filter((m) => m.server_name.toLowerCase().includes(q));
  }

  attachSkill(id: string): void {
    if (!this.agentForm.attached_skills) {
      this.agentForm.attached_skills = [];
    }
    if (id && !this.agentForm.attached_skills.includes(id)) {
      this.agentForm.attached_skills.push(id);
      this.skillSearchQuery = '';
    }
  }

  detachSkill(id: string): void {
    if (this.agentForm.attached_skills) {
      this.agentForm.attached_skills = this.agentForm.attached_skills.filter(
        (i: string) => i !== id,
      );
    }
  }

  attachTool(id: string): void {
    if (!this.agentForm.attached_tools) {
      this.agentForm.attached_tools = [];
    }
    if (id && !this.agentForm.attached_tools.includes(id)) {
      this.agentForm.attached_tools.push(id);
      this.toolSearchQuery = '';
    }
  }

  detachTool(id: string): void {
    if (this.agentForm.attached_tools) {
      this.agentForm.attached_tools = this.agentForm.attached_tools.filter(
        (i: string) => i !== id,
      );
    }
  }

  getAvailableAgentsToAttach(): any[] {
    return this.agents.filter(
      (a) =>
        a.id !== this.agentForm.id &&
        !(this.agentForm.attached_agents || []).includes(a.id || ''),
    );
  }

  getFilteredAvailableAgents(): any[] {
    const q = this.agentSearchQuery.toLowerCase().trim();
    const available = this.getAvailableAgentsToAttach();
    if (!q) return available;
    return available.filter(
      (a) =>
        a.name.toLowerCase().includes(q) ||
        (a.description && a.description.toLowerCase().includes(q)),
    );
  }

  attachAgent(id: string): void {
    if (!this.agentForm.attached_agents) {
      this.agentForm.attached_agents = [];
    }
    if (id && !this.agentForm.attached_agents.includes(id)) {
      this.agentForm.attached_agents.push(id);
      this.agentSearchQuery = '';
    }
  }

  detachAgent(id: string): void {
    if (this.agentForm.attached_agents) {
      this.agentForm.attached_agents = this.agentForm.attached_agents.filter(
        (i: string) => i !== id,
      );
    }
  }

  getAgentName(id: string): string {
    const a = this.agents.find((x) => x.id === id);
    return a ? a.name : id;
  }

  addSubAgent(): void {
    if (this.selectedSubAgentToAdd && this.agentForm) {
      if (!this.agentForm.attached_agents) this.agentForm.attached_agents = [];
      if (
        !this.agentForm.attached_agents.includes(this.selectedSubAgentToAdd)
      ) {
        this.agentForm.attached_agents.push(this.selectedSubAgentToAdd);
      }
      this.selectedSubAgentToAdd = '';
    }
  }

  removeSubAgent(agentId: string): void {
    if (this.agentForm?.attached_agents) {
      this.agentForm.attached_agents = this.agentForm.attached_agents.filter(
        (a: string) => a !== agentId,
      );
    }
  }

  addSkill(): void {
    if (this.selectedSkillToAdd && this.agentForm) {
      if (!this.agentForm.attached_skills) this.agentForm.attached_skills = [];
      if (!this.agentForm.attached_skills.includes(this.selectedSkillToAdd)) {
        this.agentForm.attached_skills.push(this.selectedSkillToAdd);
      }
      this.selectedSkillToAdd = '';
    }
  }

  removeSkill(skillId: string): void {
    if (this.agentForm?.attached_skills) {
      this.agentForm.attached_skills = this.agentForm.attached_skills.filter(
        (s: string) => s !== skillId,
      );
    }
  }

  attachTraitFromCatalog(trait: string): void {
    if (this.agentForm) {
      if (!this.agentForm.implements_traits)
        this.agentForm.implements_traits = [];
      if (!this.agentForm.implements_traits.includes(trait)) {
        this.agentForm.implements_traits.push(trait);
        this.traitSearchQuery = '';
      }
    }
  }

  attachUsesTraitFromCatalog(trait: string): void {
    if (this.agentForm) {
      if (!this.agentForm.uses_traits) this.agentForm.uses_traits = [];
      if (!this.agentForm.uses_traits.includes(trait)) {
        this.agentForm.uses_traits.push(trait);
        this.usesTraitSearchQuery = '';
      }
    }
  }

  addTrait(): void {
    if (this.newTrait.trim() && this.agentForm) {
      const traitName = this.newTrait.trim();
      if (!this.agentForm.implements_traits)
        this.agentForm.implements_traits = [];
      if (!this.agentForm.implements_traits.includes(traitName)) {
        this.agentForm.implements_traits.push(traitName);
      }
      if (!this.registeredTraitsCatalog.includes(traitName)) {
        this.registeredTraitsCatalog.push(traitName);
      }
      this.newTrait = '';
    }
  }

  selectTraitContract(trait: TraitContract): void {
    this.selectedTraitContract = trait;
    this.traitForm = { ...trait };
  }

  createNewTraitContract(): void {
    this.selectedTraitContract = null;
    this.traitForm = {
      name: 'NewTraitDefinition',
      description:
        'Describe the purpose and domain expectations of this agent trait...',
      version: '1.0.0',
      capability_requirements: [],
      behavioral_invariants: [],
      evaluation_criteria: [],
      tags: ['trait'],
    };
  }

  saveTraitContract(): void {
    if (!this.traitForm.name) return;
    const existingIdx = this.traitContracts.findIndex(
      (t) =>
        t.id === this.selectedTraitContract?.id ||
        t.name === this.traitForm.name,
    );
    if (existingIdx >= 0) {
      const v = this.traitContracts[existingIdx].version || '1.0.0';
      const parts = v.split('.');
      const minor = parts.length >= 2 ? parseInt(parts[1], 10) + 1 : 1;
      const bumpedVersion = `${parts[0] || '1'}.${minor}.0`;
      this.traitContracts[existingIdx] = {
        ...this.traitContracts[existingIdx],
        ...this.traitForm,
        version: bumpedVersion,
      } as TraitContract;
      this.selectedTraitContract = this.traitContracts[existingIdx];
      this.snackBar.open(
        `Updated trait ${this.traitForm.name} (v${this.traitContracts[existingIdx].version})`,
        'Close',
        { duration: 3000 },
      );
    } else {
      const newTrait: TraitContract = {
        id: 'trait-' + Date.now(),
        owner_id: '00000000-0000-0000-0000-000000000000',
        name: this.traitForm.name,
        description: this.traitForm.description || '',
        version: '1.0.0',
        capability_requirements: this.traitForm.capability_requirements || [],
        behavioral_invariants: this.traitForm.behavioral_invariants || [],
        evaluation_criteria: this.traitForm.evaluation_criteria || [],
        tags: this.traitForm.tags || ['trait'],
      };
      this.traitContracts.push(newTrait);
      if (!this.registeredTraitsCatalog.includes(newTrait.name)) {
        this.registeredTraitsCatalog.push(newTrait.name);
      }
      this.selectedTraitContract = newTrait;
      this.snackBar.open(`Created new trait ${newTrait.name}`, 'Close', {
        duration: 3000,
      });
    }
  }

  removeTrait(trait: string): void {
    if (this.agentForm?.implements_traits) {
      this.agentForm.implements_traits =
        this.agentForm.implements_traits.filter((t: string) => t !== trait);
    }
  }

  removeUsesTrait(trait: string): void {
    if (this.agentForm?.uses_traits) {
      this.agentForm.uses_traits = this.agentForm.uses_traits.filter(
        (t: string) => t !== trait,
      );
    }
  }

  getFilteredAgents(): Agent[] {
    const query = this.searchQuery.toLowerCase().trim();
    if (!query) return this.agents;
    return this.agents.filter(
      (a) =>
        a.name.toLowerCase().includes(query) ||
        (a.description && a.description.toLowerCase().includes(query)) ||
        (a.tags && a.tags.some((t) => t.toLowerCase().includes(query))) ||
        (a.implements_traits &&
          a.implements_traits.some((t) => t.toLowerCase().includes(query))),
    );
  }

  getRenderedMarkdown(text: string): SafeHtml {
    if (!text) return '';
    try {
      const rawHtml = marked.parse(text) as string;
      return this.sanitizer.bypassSecurityTrustHtml(rawHtml);
    } catch (e) {
      return text;
    }
  }

  syncEmbeddings() {
    if (this.selectedAgent?.id) {
      this.apiService
        .syncEmbeddings('agents', this.selectedAgent.id)
        .subscribe({
          next: (res) => {
            this.snackBar.open(
              `Synced ${res.embeddings_created} embeddings`,
              'Close',
              { duration: 3000 },
            );
          },
          error: (err) => {
            this.snackBar.open('Failed to sync embeddings', 'Close', {
              duration: 3000,
            });
          },
        });
    }
  }

  enableEdit(): void {
    this.isEditing = true;
    this.router.navigate([], {
      relativeTo: this.route,
      queryParams: { edit: 'true' },
      queryParamsHandling: 'merge'
    });
  }
}
