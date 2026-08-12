import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSelectModule } from '@angular/material/select';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatTabsModule } from '@angular/material/tabs';
import { ApiService, TraitContract } from '../../services/api.service';

@Component({
  selector: 'app-traits-registry',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatChipsModule,
    MatTooltipModule,
    MatSelectModule,
    MatTabsModule,
    MatSnackBarModule
  ],

  templateUrl: './traits-registry.component.html',
  styleUrl: './traits-registry.component.scss'
})
export class TraitsRegistryComponent implements OnInit {
  // Guardrail Catalogs for Selection
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

  traitContracts: TraitContract[] = [
    {
      id: 'trait-sec-1',
      name: 'SecurityAuditor',
      description: 'Trait for automated OWASP vulnerability scanning and memory safety auditing.',
      version: 2,
      capability_requirements: [
        'Read access to workspace source code repository and AST parser',
        'Tool access: Clippy static analyzer and OWASP dependency scanner',
        'Execution permission to run read-only static analysis passes'
      ],
      behavioral_invariants: [
        'MUST NEVER execute untrusted target code binaries during audit',
        'MUST ALWAYS report exact file paths and line ranges for discovered findings',
        'MUST NOT attempt network calls outside authorized vulnerability databases'
      ],
      evaluation_criteria: [
        'Zero false negatives on known OWASP Top 10 vulnerability test fixtures',
        'Findings must include actionable remediation advice and code snippets',
        'Precision score >= 0.90 on synthetic benchmark security suites'
      ],
      tags: ['security', 'owasp', 'audit'],
      guardrails: {
        input_guardrails: {
          active_guardrails: [
            {
              id: 'tg-1',
              type: 'prompt_injection',
              name: 'Prompt Injection Interceptor',
              tier: 'Tier 1: Deterministic',
              description: 'Real-time heuristic scanning to intercept injection signatures before reaching LLM',
              config: {}
            },
            {
              id: 'tg-2',
              type: 'pii_regex',
              name: 'PII Regex Filtering',
              tier: 'Tier 1: Deterministic',
              description: 'Regex scanning to block SSNs, emails, credit cards, and secret credentials',
              config: {}
            }
          ]
        },
        output_guardrails: {
          active_guardrails: [
            {
              id: 'tog-1',
              type: 'secret_redaction',
              name: 'Secret & API Key Redaction',
              tier: 'Data Safety & Privacy',
              description: 'Automatically scan and mask credentials, RSA private keys, and API tokens in responses',
              config: {}
            },
            {
              id: 'tog-2',
              type: 'infra_leakage_filter',
              name: 'Internal Infra & Network Leakage Filter',
              tier: 'Data Safety & Privacy',
              description: 'Redact internal stack traces, cluster IPs, private domain names, and database URIs',
              config: {}
            }
          ]
        }
      }
    },
    {
      id: 'trait-cr-1',
      name: 'CodeReviewer',
      description: 'Trait for automated PR diff inspection and code style verification.',
      version: 1,
      capability_requirements: [
        'Tool access: Git Diff Inspector and syntax tree parser',
        'Read access to PR patch file and target branch baseline'
      ],
      behavioral_invariants: [
        'MUST NEVER approve code containing syntax or compiler errors',
        'MUST ALWAYS check diff line lengths and formatting standards',
        'MUST provide constructive suggestions for code complexity reduction'
      ],
      evaluation_criteria: [
        'Comment relevance score evaluated by senior developer rubric',
        '100% detection of style guide violations in benchmark pull requests'
      ],
      tags: ['code-review', 'git'],
      guardrails: {
        input_guardrails: {
          active_guardrails: [
            {
              id: 'tg-3',
              type: 'blocked_keywords',
              name: 'Blocked Input Keyword Blacklist',
              tier: 'Tier 1: Deterministic',
              description: 'Reject exact matching phrases or regex keywords in prompts',
              config: { blocked_input_keywords: ['ignore previous instructions'] }
            }
          ]
        },
        output_guardrails: {
          active_guardrails: [
            {
              id: 'tog-3',
              type: 'structural_formatting_rules',
              name: 'Custom Structural & Regex Formatting Rules',
              tier: 'Quality & Structure',
              description: 'Validate Markdown formatting, custom code block structures, or mandatory URL patterns',
              config: {}
            }
          ]
        }
      }
    },
    {
      id: 'trait-comp-1',
      name: 'Compiler',
      description: 'Trait for validating DAG topologies and sub-agent trait compatibility.',
      version: 1,
      capability_requirements: [
        'State access: Sub-agent topology graph and trait catalog registry',
        'Tool access: Cycle detection graph traverser'
      ],
      behavioral_invariants: [
        'MUST NEVER allow circular dependencies between sub-agent execution nodes',
        'MUST ALWAYS verify that required sub-agent capability traits are fulfilled'
      ],
      evaluation_criteria: [
        'Correct classification of valid DAG topologies vs cyclic graphs',
        'Clear error diagnostics indicating broken contract node links'
      ],
      tags: ['compiler', 'dag'],
      guardrails: {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: {
          active_guardrails: [
            {
              id: 'tog-4',
              type: 'enforce_json_schema',
              name: 'Strict JSON Schema Contract Enforcement',
              tier: 'Quality & Structure',
              description: 'Validate output against formal JSON Schema contracts prior to returning payload',
              config: {}
            }
          ]
        }
      }
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
    tags: ['trait'],
    guardrails: {
      input_guardrails: { active_guardrails: [] },
      output_guardrails: { active_guardrails: [] }
    }
  };

  selectedGuardrailTypeToAdd: string = '';
  selectedOutputGuardrailTypeToAdd: string = '';
  searchQuery: string = '';
  newRequirement: string = '';
  newInvariant: string = '';
  newCriterion: string = '';
  newTag: string = '';

  constructor(private snackBar: MatSnackBar) {}

  ngOnInit(): void {
    if (this.traitContracts.length > 0) {
      this.selectTraitContract(this.traitContracts[0]);
    }
  }

  get filteredTraitContracts(): TraitContract[] {
    if (!this.searchQuery.trim()) {
      return this.traitContracts;
    }
    const q = this.searchQuery.toLowerCase().trim();
    return this.traitContracts.filter(t => 
      t.name.toLowerCase().includes(q) || 
      t.description.toLowerCase().includes(q) ||
      t.tags.some(tag => tag.toLowerCase().includes(q))
    );
  }

  selectTraitContract(trait: TraitContract): void {
    this.selectedTraitContract = trait;
    this.traitForm = {
      ...trait,
      capability_requirements: trait.capability_requirements ? [...trait.capability_requirements] : [],
      behavioral_invariants: trait.behavioral_invariants ? [...trait.behavioral_invariants] : [],
      evaluation_criteria: trait.evaluation_criteria ? [...trait.evaluation_criteria] : [],
      tags: trait.tags ? [...trait.tags] : [],
      guardrails: trait.guardrails || {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      }
    };
    if (!this.traitForm.guardrails?.input_guardrails?.active_guardrails) {
      if (!this.traitForm.guardrails) {
        this.traitForm.guardrails = {
          input_guardrails: { active_guardrails: [] },
          output_guardrails: { active_guardrails: [] }
        };
      }
      this.traitForm.guardrails.input_guardrails = { active_guardrails: [] };
    }
    if (!this.traitForm.guardrails?.output_guardrails?.active_guardrails) {
      this.traitForm.guardrails.output_guardrails = { active_guardrails: [] };
    }
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
      tags: ['trait'],
      guardrails: {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      }
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
        tags: this.traitForm.tags || ['trait'],
        guardrails: this.traitForm.guardrails || {
          input_guardrails: { active_guardrails: [] },
          output_guardrails: { active_guardrails: [] }
        }
      };
      this.traitContracts.push(newTrait);
      this.selectedTraitContract = newTrait;
      this.snackBar.open(`Created new trait ${newTrait.name}`, 'Close', { duration: 3000 });
    }
  }

  // Trait Guardrails Management
  addInputGuardrailFromDropdown(): void {
    if (!this.selectedGuardrailTypeToAdd) return;
    const catItem = this.guardrailCatalog.find(g => g.type === this.selectedGuardrailTypeToAdd);
    if (!catItem) return;

    if (!this.traitForm.guardrails) {
      this.traitForm.guardrails = {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      };
    }
    if (!this.traitForm.guardrails.input_guardrails) {
      this.traitForm.guardrails.input_guardrails = { active_guardrails: [] };
    }

    const newGuardrail: any = {
      id: 'tg-' + Date.now(),
      type: catItem.type,
      name: catItem.name,
      tier: catItem.tier,
      description: catItem.description,
      config: {}
    };

    this.traitForm.guardrails.input_guardrails.active_guardrails.push(newGuardrail);
    this.selectedGuardrailTypeToAdd = '';
    this.snackBar.open(`Added trait input guardrail ${catItem.name}`, 'Close', { duration: 2500 });
  }

  deleteInputGuardrail(id: string): void {
    if (this.traitForm.guardrails?.input_guardrails?.active_guardrails) {
      this.traitForm.guardrails.input_guardrails.active_guardrails = 
        this.traitForm.guardrails.input_guardrails.active_guardrails.filter(g => g.id !== id);
      this.snackBar.open('Trait input guardrail removed', 'Close', { duration: 2000 });
    }
  }

  addOutputGuardrailFromDropdown(): void {
    if (!this.selectedOutputGuardrailTypeToAdd) return;
    const catItem = this.outputGuardrailCatalog.find(g => g.type === this.selectedOutputGuardrailTypeToAdd);
    if (!catItem) return;

    if (!this.traitForm.guardrails) {
      this.traitForm.guardrails = {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      };
    }
    if (!this.traitForm.guardrails.output_guardrails) {
      this.traitForm.guardrails.output_guardrails = { active_guardrails: [] };
    }

    const newGuardrail: any = {
      id: 'tog-' + Date.now(),
      type: catItem.type,
      name: catItem.name,
      tier: catItem.category,
      description: catItem.description,
      config: {}
    };

    this.traitForm.guardrails.output_guardrails.active_guardrails.push(newGuardrail);
    this.selectedOutputGuardrailTypeToAdd = '';
    this.snackBar.open(`Added trait output guardrail ${catItem.name}`, 'Close', { duration: 2500 });
  }

  deleteOutputGuardrail(id: string): void {
    if (this.traitForm.guardrails?.output_guardrails?.active_guardrails) {
      this.traitForm.guardrails.output_guardrails.active_guardrails = 
        this.traitForm.guardrails.output_guardrails.active_guardrails.filter(g => g.id !== id);
      this.snackBar.open('Trait output guardrail removed', 'Close', { duration: 2000 });
    }
  }

  // Capability Requirements Helper Methods
  addRequirement(): void {
    if (this.newRequirement.trim()) {
      if (!this.traitForm.capability_requirements) this.traitForm.capability_requirements = [];
      this.traitForm.capability_requirements.push(this.newRequirement.trim());
      this.newRequirement = '';
    }
  }

  removeRequirement(req: string): void {
    if (this.traitForm.capability_requirements) {
      this.traitForm.capability_requirements = this.traitForm.capability_requirements.filter(r => r !== req);
    }
  }

  // Behavioral Invariants Helper Methods
  addInvariant(): void {
    if (this.newInvariant.trim()) {
      if (!this.traitForm.behavioral_invariants) this.traitForm.behavioral_invariants = [];
      this.traitForm.behavioral_invariants.push(this.newInvariant.trim());
      this.newInvariant = '';
    }
  }

  removeInvariant(inv: string): void {
    if (this.traitForm.behavioral_invariants) {
      this.traitForm.behavioral_invariants = this.traitForm.behavioral_invariants.filter(i => i !== inv);
    }
  }

  // Evaluation Criteria Helper Methods
  addCriterion(): void {
    if (this.newCriterion.trim()) {
      if (!this.traitForm.evaluation_criteria) this.traitForm.evaluation_criteria = [];
      this.traitForm.evaluation_criteria.push(this.newCriterion.trim());
      this.newCriterion = '';
    }
  }

  removeCriterion(crit: string): void {
    if (this.traitForm.evaluation_criteria) {
      this.traitForm.evaluation_criteria = this.traitForm.evaluation_criteria.filter(c => c !== crit);
    }
  }

  // Tags Helper Methods
  addTag(): void {
    if (this.newTag.trim()) {
      if (!this.traitForm.tags) this.traitForm.tags = [];
      if (!this.traitForm.tags.includes(this.newTag.trim())) {
        this.traitForm.tags.push(this.newTag.trim());
      }
      this.newTag = '';
    }
  }

  removeTag(tag: string): void {
    if (this.traitForm.tags) {
      this.traitForm.tags = this.traitForm.tags.filter(t => t !== tag);
    }
  }
}
