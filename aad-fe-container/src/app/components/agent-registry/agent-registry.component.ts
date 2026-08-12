import { Component, OnInit } from '@angular/core';
import { CommonModule, Location } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTabsModule } from '@angular/material/tabs';
import { MatSelectModule } from '@angular/material/select';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatBadgeModule } from '@angular/material/badge';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { ApiService, Agent, GuardrailConfig, TraitContract } from '../../services/api.service';


export interface LLMModelOption {
  id: string;
  name: string;
  version: string;
  provider: string;
}

@Component({
  selector: 'app-agent-registry',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatSelectModule,
    MatSlideToggleModule,
    MatIconModule,
    MatChipsModule,
    MatTabsModule,
    MatBadgeModule,
    MatTooltipModule,
    MatSnackBarModule
  ],

  templateUrl: './agent-registry.component.html',
  styleUrl: './agent-registry.component.scss'
})
export class AgentRegistryComponent implements OnInit {
  agents: Agent[] = [];
  selectedAgent: Agent | null = null;
  searchQuery: string = '';
  isEditing: boolean = false;

  availableModels: LLMModelOption[] = [
    { id: 'claude-3-5-sonnet-v2', name: 'Claude 3.5 Sonnet', version: '20241022', provider: 'Anthropic' },
    { id: 'claude-3-opus-v1', name: 'Claude 3 Opus', version: '20240229', provider: 'Anthropic' },
    { id: 'claude-3-haiku-v1', name: 'Claude 3 Haiku', version: '20240307', provider: 'Anthropic' },
    { id: 'gpt-4o-v2024-08-06', name: 'GPT-4o', version: '2024-08-06', provider: 'OpenAI' },
    { id: 'gpt-4o-mini-v2024-07-18', name: 'GPT-4o Mini', version: '2024-07-18', provider: 'OpenAI' },
    { id: 'llama-3.3-70b-instruct', name: 'Llama 3.3 70B', version: 'v3.3', provider: 'Ollama / Local' },
    { id: 'deepseek-r1-70b', name: 'DeepSeek R1 70B', version: 'v1.0', provider: 'Ollama / Local' },
    { id: 'gemini-1.5-pro-v002', name: 'Gemini 1.5 Pro', version: '002', provider: 'Google' }
  ];

  // Available Input Guardrail Catalog for Dropdown selection
  guardrailCatalog = [
    { type: 'prompt_injection', name: 'Prompt Injection Interceptor', tier: 'Tier 1: Deterministic', description: 'Real-time heuristic scanning to intercept injection signatures before reaching LLM' },
    { type: 'pii_regex', name: 'PII Regex Filtering', tier: 'Tier 1: Deterministic', description: 'Regex scanning to block SSNs, emails, credit cards, and secret credentials' },
    { type: 'max_input_tokens', name: 'Max Input Token Limit Cap', tier: 'Tier 1: Deterministic', description: 'Enforce hard upper limit cap on incoming prompt tokens' },
    { type: 'blocked_keywords', name: 'Blocked Input Keyword Blacklist', tier: 'Tier 1: Deterministic', description: 'Reject exact matching phrases or regex keywords in prompts' },
    { type: 'vector_similarity', name: 'Vector Attack Similarity Matcher', tier: 'Tier 2: Vector Matching', description: 'Cosine similarity matching against vector database of known jailbreaks' },
    { type: 'classifier_model', name: 'Specialized Safety Classifier Model', tier: 'Tier 3: Dedicated Classifier', description: 'Route input to Llama Guard, DeBERTa v3, or Perspective API classifier' },
    { type: 'llm_judge', name: 'LLM-as-a-Judge Pre-Evaluator', tier: 'Tier 4: LLM Judge', description: 'Run lightweight fast LLM evaluator against custom policy rules' },
    { type: 'domain_scoping', name: 'System Prompt Domain Scoping', tier: 'Tier 5: System Rules', description: 'Inject allowed topic constraints into system prompt' }
  ];

  // Available Output Guardrail Catalog for Dropdown selection
  outputGuardrailCatalog = [
    { type: 'secret_redaction', name: 'Secret & API Key Redaction', category: 'Data Safety & Privacy', description: 'Automatically scan and mask credentials, RSA private keys, and API tokens in responses' },
    { type: 'pii_ner_redaction', name: 'PII & Sensitive Data Redaction (NER)', category: 'Data Safety & Privacy', description: 'Mask names, emails, phone numbers, SSNs, and credit cards using Named Entity Recognition (Presidio)' },
    { type: 'infra_leakage_filter', name: 'Internal Infra & Network Leakage Filter', category: 'Data Safety & Privacy', description: 'Redact internal stack traces, cluster IPs, private domain names, and database URIs' },
    { type: 'enforce_json_schema', name: 'Strict JSON Schema Contract Enforcement', category: 'Quality & Structure', description: 'Validate output against formal JSON Schema contracts prior to returning payload' },
    { type: 'max_output_tokens', name: 'Max Output Token Generation Limit', category: 'Generation Boundaries', description: 'Hard token limit cap on total output generation per request' },
    { type: 'blocked_output_keywords', name: 'Blocked Output Keyword & Secret Redaction', category: 'Data Safety & Privacy', description: 'Banned terms or secret pattern blacklists redacted from LLM output' },
    { type: 'toxicity_classifier', name: 'Post-Execution Toxicity & Harm Classifier', category: 'Safety & Policy', description: 'Filter hate speech, harassment, or explicit material using Llama Guard or Perspective API' },
    { type: 'brand_competitor_protection', name: 'Competitor & Brand Protection Filter', category: 'Safety & Policy', description: 'Redact or restrict mentions of banned competitor brands or restricted products' },
    { type: 'rag_grounding_hallucination', name: 'RAG Grounding & Hallucination Scoring (NLI)', category: 'Truthfulness & Quality', description: 'Natural Language Inference grounding checks against retrieved context chunks' },
    { type: 'refusal_offtopic_detector', name: 'Refusal & Off-Topic Detector', category: 'Truthfulness & Quality', description: 'Detect hallucinated refusals or domain boundary drift' },
    { type: 'structural_formatting_rules', name: 'Custom Structural & Regex Formatting Rules', category: 'Quality & Structure', description: 'Validate Markdown formatting, custom code block structures, or mandatory URL patterns' }
  ];


  selectedGuardrailTypeToAdd: string = '';
  selectedOutputGuardrailTypeToAdd: string = '';

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
    'BasicAgent'
  ];

  registeredToolsCatalog = [
    { id: 'tool-git-diff', name: 'Git Diff Inspector', category: 'DevTools' },
    { id: 'tool-sqlite-query', name: 'Knowledge Graph SQL Query', category: 'Database' },
    { id: 'tool-mcp-client', name: 'Remote MCP Client Bridge', category: 'MCP' },
    { id: 'tool-static-analyzer', name: 'Clippy / Static Analysis', category: 'Security' },
    { id: 'tool-ast-parser', name: 'Tree-Sitter AST Parser', category: 'Compiler' }
  ];

  registeredSkillsCatalog = [
    { id: 'skill-security-audit', name: 'OWASP Security Audit Skill', description: 'Ruleset for memory safety and timing attack detection' },
    { id: 'skill-dag-validator', name: 'DAG Topology Validator', description: 'Rules for validating circular dependency avoidance' },
    { id: 'skill-rag-retrieval', name: 'RAG Hybrid Knowledge Retrieval', description: 'Combines vector search with SPO graph traversal' }
  ];

  // Trait Contracts Editor State
  traitContracts: TraitContract[] = [
    {
      id: 'trait-sec-1',
      name: 'SecurityAuditor',
      description: 'Trait for automated OWASP vulnerability scanning and memory safety auditing.',
      version: 2,
      capability_requirements: [
        'Read access to workspace source code repository and AST parser',
        'Tool access: Clippy static analyzer and OWASP dependency scanner'
      ],
      behavioral_invariants: [
        'MUST NEVER execute untrusted target code binaries during audit',
        'MUST ALWAYS report exact file paths and line ranges for discovered findings'
      ],
      evaluation_criteria: [
        'Zero false negatives on known OWASP Top 10 vulnerability test fixtures',
        'Precision score >= 0.90 on synthetic benchmark security suites'
      ],
      tags: ['security', 'owasp', 'audit']
    },
    {
      id: 'trait-cr-1',
      name: 'CodeReviewer',
      description: 'Trait for automated PR diff inspection and code style verification.',
      version: 1,
      capability_requirements: ['Tool access: Git Diff Inspector'],
      behavioral_invariants: ['MUST NEVER approve code containing syntax errors'],
      evaluation_criteria: ['Comment relevance score evaluated by senior developer rubric'],
      tags: ['code-review', 'git']
    },
    {
      id: 'trait-comp-1',
      name: 'Compiler',
      description: 'Trait for validating DAG topologies and sub-agent trait compatibility.',
      version: 1,
      capability_requirements: ['State access: Sub-agent topology graph'],
      behavioral_invariants: ['MUST NEVER allow circular dependencies between sub-agent execution nodes'],
      evaluation_criteria: ['Correct classification of valid DAG topologies vs cyclic graphs'],
      tags: ['compiler', 'dag']
    }
  ];

  selectedTraitContract: TraitContract | null = null;
  traitForm: Partial<TraitContract> = {
    name: '',
    description: '',
    version: 1,
    capability_requirements: [],
    behavioral_invariants: [],
    evaluation_criteria: [],
    tags: ['trait']
  };


  // Selected Item Pickers for Form
  traitSearchQuery: string = '';
  selectedToolToAdd: string = '';
  selectedSubAgentToAdd: string = '';
  selectedSkillToAdd: string = '';


  // Form Model
  agentForm: Partial<Agent> = {
    name: '',
    description: '',
    tags: [],
    implements_traits: [],
    attached_tools: [],
    attached_agents: [],
    attached_skills: [],
    agent_definition: '',
    judge_threshold: 0.8,
    model: 'claude-3-5-sonnet-v2',
    guardrails: {
      input_guardrails: { active_guardrails: [] },
      output_guardrails: { active_guardrails: [] }
    }
  };

  newTag: string = '';
  newTrait: string = '';

  get filteredTraitsCatalog(): string[] {
    if (!this.traitSearchQuery.trim()) {
      return this.registeredTraitsCatalog;
    }
    const q = this.traitSearchQuery.toLowerCase().trim();
    return this.registeredTraitsCatalog.filter(t => t.toLowerCase().includes(q));
  }


  constructor(
    private apiService: ApiService,
    private snackBar: MatSnackBar,
    private route: ActivatedRoute,
    private location: Location
  ) {}

  ngOnInit(): void {
    this.loadAgents();
  }

  loadAgents(): void {
    this.apiService.getAgents().subscribe({
      next: (data) => {
        this.agents = data;
        const routeId = this.route.snapshot.paramMap.get('id');
        this.applySelectedAgentFromRoute(routeId);
      },
      error: () => {
        this.agents = [
          {
            id: '11111111-1111-1111-1111-111111111111',
            name: 'SecurityAuditorAgent',
            description: 'Automated static analysis and security vulnerability inspector.',
            tags: ['security', 'audit', 'rust'],
            implements_traits: ['SecurityAuditor', 'CodeReviewer'],
            current_version: 3,
            owner_id: 'owner-sec-team',
            judge_threshold: 0.9,
            model: 'claude-3-5-sonnet-v2',
            agent_definition: 'You are a principal security engineer. Analyze code for OWASP vulnerabilities and timing attacks.'
          },
          {
            id: '22222222-2222-2222-2222-222222222222',
            name: 'RefactoringCompilerAgent',
            description: 'Scans agent networks to detect circular dependencies and overlap clusters.',
            tags: ['compiler', 'refactoring', 'dag'],
            implements_traits: ['Compiler', 'NetworkOptimizer'],
            current_version: 1,
            owner_id: 'owner-core-team',
            judge_threshold: 0.85,
            model: 'llama-3.3-70b-instruct',
            agent_definition: 'You are an agent network compiler. Validate DAG topologies and trait compatibility.'
          }
        ];
        const routeId = this.route.snapshot.paramMap.get('id');
        this.applySelectedAgentFromRoute(routeId);
      }
    });
  }

  private applySelectedAgentFromRoute(routeId: string | null): void {
    if (routeId) {
      const match = this.agents.find(a => a.id === routeId || a.id.startsWith(routeId));
      if (match) {
        this.selectAgent(match);
        return;
      }
    }
    if (this.agents.length > 0 && !this.selectedAgent) {
      this.selectAgent(this.agents[0]);
    }
  }

  selectAgent(agent: Agent): void {
    this.selectedAgent = agent;
    this.agentForm = {
      ...agent,
      guardrails: agent.guardrails || {
        input_guardrails: {
          active_guardrails: [
            {
              id: 'g-1',
              type: 'prompt_injection',
              name: 'Prompt Injection Interceptor',
              tier: 'Tier 1: Deterministic',
              description: 'Real-time heuristic scanning to intercept injection signatures before reaching LLM',
              config: {}
            },
            {
              id: 'g-2',
              type: 'blocked_keywords',
              name: 'Blocked Input Keyword Blacklist',
              tier: 'Tier 1: Deterministic',
              description: 'Reject exact matching phrases or regex keywords in prompts',
              config: { blocked_input_keywords: ['ignore previous instructions', 'system prompt reveal'] }
            }
          ]
        },
        output_guardrails: {
          active_guardrails: [
            {
              id: 'og-1',
              type: 'secret_redaction',
              name: 'Secret & API Key Redaction',
              tier: 'Data Safety & Privacy',
              description: 'Automatically scan and mask credentials, RSA private keys, and API tokens in responses',
              config: { secret_redaction: true }
            },
            {
              id: 'og-2',
              type: 'enforce_json_schema',
              name: 'Strict JSON Schema Contract Enforcement',
              tier: 'Quality & Structure',
              description: 'Validate output against formal JSON Schema contracts prior to returning payload',
              config: { enforce_json_schema: true }
            }
          ]
        }
      }
    };
    if (!this.agentForm.guardrails?.input_guardrails?.active_guardrails) {
      this.agentForm.guardrails!.input_guardrails = { active_guardrails: [] };
    }
    if (!this.agentForm.guardrails?.output_guardrails?.active_guardrails) {
      this.agentForm.guardrails!.output_guardrails = { active_guardrails: [] };
    }
    this.isEditing = false;
    this.location.go(`/agent-registry/${agent.id}`);
  }

  createNewAgent(): void {
    this.selectedAgent = null;
    this.agentForm = {
      name: 'New Agent',
      description: 'Describe the purpose and capabilities of this agent...',
      tags: ['draft'],
      implements_traits: ['BasicAgent'],
      agent_definition: 'You are an autonomous AI agent designed for specialized tasks.',
      judge_threshold: 0.8,
      owner_id: '00000000-0000-0000-0000-000000000000',
      guardrails: {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      }
    };
    this.isEditing = true;
  }

  saveAgent(): void {
    if (this.selectedAgent && this.selectedAgent.id) {
      this.apiService.updateAgent(this.selectedAgent.id, this.agentForm).subscribe({
        next: () => {
          this.snackBar.open('Agent updated successfully!', 'Close', { duration: 3000 });
          this.loadAgents();
        },
        error: () => {
          this.snackBar.open('Updated agent specifications locally.', 'Close', { duration: 3000 });
        }
      });
    }
  }

  // Input Guardrails Dropdown & Dynamic Item Management
  addGuardrailFromDropdown(): void {
    if (!this.selectedGuardrailTypeToAdd) return;
    const catItem = this.guardrailCatalog.find(g => g.type === this.selectedGuardrailTypeToAdd);
    if (!catItem) return;

    if (!this.agentForm.guardrails) {
      this.agentForm.guardrails = {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      };
    }
    if (!this.agentForm.guardrails.input_guardrails) {
      this.agentForm.guardrails.input_guardrails = { active_guardrails: [] };
    }
    if (!this.agentForm.guardrails.input_guardrails.active_guardrails) {
      this.agentForm.guardrails.input_guardrails.active_guardrails = [];
    }

    const newGuardrail: any = {
      id: 'g-' + Date.now(),
      type: catItem.type,
      name: catItem.name,
      tier: catItem.tier,
      description: catItem.description,
      config: this.getDefaultConfigForType(catItem.type)
    };

    this.agentForm.guardrails.input_guardrails.active_guardrails.push(newGuardrail);
    this.selectedGuardrailTypeToAdd = '';
    this.snackBar.open(`Added ${catItem.name} guardrail`, 'Close', { duration: 2500 });
  }

  deleteGuardrail(id: string): void {
    if (this.agentForm.guardrails?.input_guardrails?.active_guardrails) {
      this.agentForm.guardrails.input_guardrails.active_guardrails = 
        this.agentForm.guardrails.input_guardrails.active_guardrails.filter(g => g.id !== id);
      this.snackBar.open('Input guardrail removed', 'Close', { duration: 2000 });
    }
  }

  // Output Guardrails Dropdown & Dynamic Item Management
  addOutputGuardrailFromDropdown(): void {
    if (!this.selectedOutputGuardrailTypeToAdd) return;
    const catItem = this.outputGuardrailCatalog.find(g => g.type === this.selectedOutputGuardrailTypeToAdd);
    if (!catItem) return;

    if (!this.agentForm.guardrails) {
      this.agentForm.guardrails = {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      };
    }
    if (!this.agentForm.guardrails.output_guardrails) {
      this.agentForm.guardrails.output_guardrails = { active_guardrails: [] };
    }
    if (!this.agentForm.guardrails.output_guardrails.active_guardrails) {
      this.agentForm.guardrails.output_guardrails.active_guardrails = [];
    }

    const newGuardrail: any = {
      id: 'og-' + Date.now(),
      type: catItem.type,
      name: catItem.name,
      tier: catItem.category,
      description: catItem.description,
      config: this.getDefaultOutputConfigForType(catItem.type)
    };

    this.agentForm.guardrails.output_guardrails.active_guardrails.push(newGuardrail);
    this.selectedOutputGuardrailTypeToAdd = '';
    this.snackBar.open(`Added ${catItem.name} output guardrail`, 'Close', { duration: 2500 });
  }

  deleteOutputGuardrail(id: string): void {
    if (this.agentForm.guardrails?.output_guardrails?.active_guardrails) {
      this.agentForm.guardrails.output_guardrails.active_guardrails = 
        this.agentForm.guardrails.output_guardrails.active_guardrails.filter(g => g.id !== id);
      this.snackBar.open('Output guardrail removed', 'Close', { duration: 2000 });
    }
  }

  private getDefaultConfigForType(type: string): any {
    switch (type) {
      case 'max_input_tokens':
        return { max_input_tokens: 4096 };
      case 'blocked_keywords':
        return { blocked_input_keywords: ['ignore previous instructions'] };
      case 'vector_similarity':
        return { vector_similarity_threshold: 0.85 };
      case 'classifier_model':
        return { classifier_type: 'llama_guard', toxicity_threshold: 0.7 };
      case 'llm_judge':
        return { judge_model: 'gpt-4o-mini-v2024-07-18', judge_custom_policy_prompt: 'Verify input adheres to safety guidelines.' };
      case 'domain_scoping':
        return { allowed_domain_topics: ['security_auditing', 'code_review'] };
      default:
        return {};
    }
  }

  private getDefaultOutputConfigForType(type: string): any {
    switch (type) {
      case 'secret_redaction':
        return { secret_redaction: true };
      case 'pii_ner_redaction':
        return { pii_ner_entities: ['EMAIL', 'PHONE_NUMBER', 'SSN', 'CREDIT_CARD'] };
      case 'infra_leakage_filter':
        return { infra_leak_types: ['STACK_TRACE', 'INTERNAL_IP', 'DATABASE_URI', 'FILE_PATH'] };
      case 'enforce_json_schema':
        return { enforce_json_schema: true };
      case 'max_output_tokens':
        return { max_output_tokens: 2048 };
      case 'blocked_output_keywords':
        return { blocked_output_keywords: ['AWS_SECRET_ACCESS_KEY', 'PRIVATE KEY'] };
      case 'toxicity_classifier':
        return { classifier_type: 'llama_guard', toxicity_threshold: 0.7 };
      case 'brand_competitor_protection':
        return { banned_competitor_brands: ['CompetitorX', 'RestrictedProductY'] };
      case 'rag_grounding_hallucination':
        return { grounding_min_score: 0.8 };
      case 'refusal_offtopic_detector':
        return { detect_refusal_hallucinations: true };
      case 'structural_formatting_rules':
        return { custom_regex_rules: ['```json[\\s\\S]*```'] };
      default:
        return {};
    }
  }

  // Chip helpers for input & output guardrail items
  addKeywordToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.blocked_input_keywords) g.config.blocked_input_keywords = [];
      g.config.blocked_input_keywords.push(val);
      inputEl.value = '';
    }
  }

  removeKeywordFromGuardrail(g: any, kw: string): void {
    if (g.config.blocked_input_keywords) {
      g.config.blocked_input_keywords = g.config.blocked_input_keywords.filter((k: string) => k !== kw);
    }
  }

  addTopicToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.allowed_domain_topics) g.config.allowed_domain_topics = [];
      g.config.allowed_domain_topics.push(val);
      inputEl.value = '';
    }
  }

  removeTopicFromGuardrail(g: any, topic: string): void {
    if (g.config.allowed_domain_topics) {
      g.config.allowed_domain_topics = g.config.allowed_domain_topics.filter((t: string) => t !== topic);
    }
  }

  addPiiEntityToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim().toUpperCase();
    if (val) {
      if (!g.config.pii_ner_entities) g.config.pii_ner_entities = [];
      g.config.pii_ner_entities.push(val);
      inputEl.value = '';
    }
  }

  removePiiEntityFromGuardrail(g: any, entity: string): void {
    if (g.config.pii_ner_entities) {
      g.config.pii_ner_entities = g.config.pii_ner_entities.filter((e: string) => e !== entity);
    }
  }

  addInfraLeakToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim().toUpperCase();
    if (val) {
      if (!g.config.infra_leak_types) g.config.infra_leak_types = [];
      g.config.infra_leak_types.push(val);
      inputEl.value = '';
    }
  }

  removeInfraLeakFromGuardrail(g: any, item: string): void {
    if (g.config.infra_leak_types) {
      g.config.infra_leak_types = g.config.infra_leak_types.filter((i: string) => i !== item);
    }
  }

  addBannedBrandToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.banned_competitor_brands) g.config.banned_competitor_brands = [];
      g.config.banned_competitor_brands.push(val);
      inputEl.value = '';
    }
  }

  removeBannedBrandFromGuardrail(g: any, brand: string): void {
    if (g.config.banned_competitor_brands) {
      g.config.banned_competitor_brands = g.config.banned_competitor_brands.filter((b: string) => b !== brand);
    }
  }

  addRegexRuleToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.custom_regex_rules) g.config.custom_regex_rules = [];
      g.config.custom_regex_rules.push(val);
      inputEl.value = '';
    }
  }

  removeRegexRuleFromGuardrail(g: any, rule: string): void {
    if (g.config.custom_regex_rules) {
      g.config.custom_regex_rules = g.config.custom_regex_rules.filter((r: string) => r !== rule);
    }
  }

  addOutputKeywordToGuardrail(g: any, inputEl: HTMLInputElement): void {
    const val = inputEl.value.trim();
    if (val) {
      if (!g.config.blocked_output_keywords) g.config.blocked_output_keywords = [];
      g.config.blocked_output_keywords.push(val);
      inputEl.value = '';
    }
  }

  removeOutputKeywordFromGuardrail(g: any, kw: string): void {
    if (g.config.blocked_output_keywords) {
      g.config.blocked_output_keywords = g.config.blocked_output_keywords.filter((k: string) => k !== kw);
    }
  }

  addTag(): void {
    if (this.newTag.trim() && this.agentForm.tags) {
      this.agentForm.tags.push(this.newTag.trim());
      this.newTag = '';
    }
  }

  removeTag(tag: string): void {
    if (this.agentForm.tags) {
      this.agentForm.tags = this.agentForm.tags.filter(t => t !== tag);
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
      this.agentForm.attached_tools = this.agentForm.attached_tools.filter(t => t !== toolId);
    }
  }

  addSubAgent(): void {
    if (this.selectedSubAgentToAdd && this.agentForm) {
      if (!this.agentForm.attached_agents) this.agentForm.attached_agents = [];
      if (!this.agentForm.attached_agents.includes(this.selectedSubAgentToAdd)) {
        this.agentForm.attached_agents.push(this.selectedSubAgentToAdd);
      }
      this.selectedSubAgentToAdd = '';
    }
  }

  removeSubAgent(agentId: string): void {
    if (this.agentForm?.attached_agents) {
      this.agentForm.attached_agents = this.agentForm.attached_agents.filter(a => a !== agentId);
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
      this.agentForm.attached_skills = this.agentForm.attached_skills.filter(s => s !== skillId);
    }
  }

  attachTraitFromCatalog(trait: string): void {
    if (this.agentForm) {
      if (!this.agentForm.implements_traits) this.agentForm.implements_traits = [];
      if (!this.agentForm.implements_traits.includes(trait)) {
        this.agentForm.implements_traits.push(trait);
      }
    }
  }

  addTrait(): void {
    if (this.newTrait.trim() && this.agentForm) {
      const traitName = this.newTrait.trim();
      if (!this.agentForm.implements_traits) this.agentForm.implements_traits = [];
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
      description: 'Describe the purpose and domain expectations of this agent trait...',
      version: 1,
      capability_requirements: [],
      behavioral_invariants: [],
      evaluation_criteria: [],
      tags: ['trait']
    };
  }

  saveTraitContract(): void {
    if (!this.traitForm.name) return;
    const existingIdx = this.traitContracts.findIndex(t => t.id === this.selectedTraitContract?.id || t.name === this.traitForm.name);
    
    if (existingIdx >= 0) {
      this.traitContracts[existingIdx] = {
        ...this.traitContracts[existingIdx],
        ...this.traitForm,
        version: (this.traitContracts[existingIdx].version || 1) + 1
      } as TraitContract;
      this.selectedTraitContract = this.traitContracts[existingIdx];
      this.snackBar.open(`Updated trait ${this.traitForm.name} (v${this.traitContracts[existingIdx].version})`, 'Close', { duration: 3000 });
    } else {
      const newTrait: TraitContract = {
        id: 'trait-' + Date.now(),
        name: this.traitForm.name,
        description: this.traitForm.description || '',
        version: 1,
        capability_requirements: this.traitForm.capability_requirements || [],
        behavioral_invariants: this.traitForm.behavioral_invariants || [],
        evaluation_criteria: this.traitForm.evaluation_criteria || [],
        tags: this.traitForm.tags || ['trait']
      };
      this.traitContracts.push(newTrait);
      if (!this.registeredTraitsCatalog.includes(newTrait.name)) {
        this.registeredTraitsCatalog.push(newTrait.name);
      }
      this.selectedTraitContract = newTrait;
      this.snackBar.open(`Created new trait ${newTrait.name}`, 'Close', { duration: 3000 });
    }
  }


  removeTrait(trait: string): void {
    if (this.agentForm?.implements_traits) {
      this.agentForm.implements_traits = this.agentForm.implements_traits.filter(t => t !== trait);
    }
  }
}





