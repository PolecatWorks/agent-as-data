import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
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
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatTabsModule } from '@angular/material/tabs';
import { MatMenuModule } from '@angular/material/menu';
import { ApiService, TraitContract } from '../../services/api.service';
import { GuardrailsEditorComponent } from '../guardrails-editor/guardrails-editor.component';
import { forkJoin } from 'rxjs';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';

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
    MatSnackBarModule,
    GuardrailsEditorComponent,
    MatFormFieldModule,
    MatMenuModule,
    RouterModule
  ],

  templateUrl: './traits-registry.component.html',
  styleUrl: './traits-registry.component.scss'
})
export class TraitsRegistryComponent implements OnInit {
  isSidebarCollapsed = false;
  menuItems = APP_NAV_MENU_ITEMS;

  toggleSidebar(): void {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  traitContracts: TraitContract[] = [
    {
      owner_id: '00000000-0000-0000-0000-000000000000',
      id: 'trait-sec-1',
      name: 'SecurityAuditor',
      description: 'Trait for automated OWASP vulnerability scanning and memory safety auditing.',
      version: '2.0.0',
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
      owner_id: '00000000-0000-0000-0000-000000000000',
      id: 'trait-cr-1',
      name: 'CodeReviewer',
      description: 'Trait for automated PR diff inspection and code style verification.',
      version: '1.0.0',
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
      owner_id: '00000000-0000-0000-0000-000000000000',
      id: 'trait-comp-1',
      name: 'Compiler',
      description: 'Trait for validating DAG topologies and sub-agent trait compatibility.',
      version: '1.0.0',
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
    version: '1.0.0',
    capability_requirements: [],
    behavioral_invariants: [],
    evaluation_criteria: [],
    tags: ['trait'],
    guardrails: {
      input_guardrails: { active_guardrails: [] },
      output_guardrails: { active_guardrails: [] }
    }
  };

  
  searchQuery: string = '';
  newRequirement: string = '';
  newInvariant: string = '';
  newCriterion: string = '';
  newTag: string = '';

  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;

  constructor(
    private snackBar: MatSnackBar,
    private apiService: ApiService,
    private route: ActivatedRoute,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadTraits();
    this.route.paramMap.subscribe(params => {
      const id = params.get('id');
      if (id) {
        this.selectTraitById(id);
      }
    });
    this.route.queryParams.subscribe(queryParams => {
      this.isEditing = queryParams['edit'] === 'true';
    });
  }

  loadTraits(): void {
    this.apiService.getTraits().subscribe({
      next: (listPages: any) => {
        const ids = listPages.ids || [];
        if (ids.length > 0) {
          const obs = ids.map((id: string) => this.apiService.getTrait(id));
          (forkJoin(obs) as any).subscribe({
            next: (fullTraits: any[]) => {
              this.traitContracts = fullTraits;
              const routeId = this.route.snapshot.paramMap.get('id');
              const isEdit = this.route.snapshot.queryParams['edit'] === 'true';
              if (routeId) {
                this.selectTraitById(routeId);
              } else if (!this.isEditing && !isEdit && !this.selectedTraitContract) {
                this.selectTraitContract(this.traitContracts[0]);
              }
            },
            error: () => {
              this.snackBar.open("Failed to load details for traits.", "Close", { duration: 3000 });
            }
          });
        } else {
          this.traitContracts = [];
          if (!this.isEditing) {
            this.selectedTraitContract = null;
          }
        }
      },
      error: () => {
        this.snackBar.open("Failed to load traits from backend, using catalog stubs.", "Close", { duration: 3000 });
        if (this.traitContracts.length > 0) {
          const id = this.route.snapshot.paramMap.get('id');
          if (id) {
            this.selectTraitById(id);
          } else if (!this.isEditing) {
            this.selectTraitContract(this.traitContracts[0]);
          }
        }
      }
    });
  }

  selectTraitById(id: string): void {
    const trait = this.traitContracts.find(t => t.id === id);
    if (trait) {
      const isEdit = this.route.snapshot.queryParams['edit'] === 'true';
      this.selectTraitContract(trait, isEdit);
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

  selectTraitContract(trait: TraitContract, keepEdit = false): void {
    this.selectedTraitContract = trait;
    this.isEditing = keepEdit;
    this.showDeleteConfirm = false;
    this.router.navigate(['/traits', trait.id], {
      queryParams: keepEdit ? { edit: 'true' } : {}
    });
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
    if (!this.traitForm.guardrails) {
      this.traitForm.guardrails = {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      };
    }
    if (!this.traitForm.guardrails.input_guardrails) {
      this.traitForm.guardrails.input_guardrails = { active_guardrails: [] };
    }
    if (!this.traitForm.guardrails.output_guardrails) {
      this.traitForm.guardrails.output_guardrails = { active_guardrails: [] };
    }
  }

  createNewTraitContract(): void {
    this.selectedTraitContract = null;
    this.isEditing = true;
    this.showDeleteConfirm = false;
    this.traitForm = {
      name: '',
      owner_id: '00000000-0000-0000-0000-000000000000',
      description: '',
      version: '1.0.0',
      capability_requirements: [],
      behavioral_invariants: [],
      evaluation_criteria: [],
      tags: [],
      guardrails: {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] }
      }
    };
    this.router.navigate(['/traits'], { queryParams: { edit: 'true' } });
  }

  enableEdit(): void {
    this.isEditing = true;
    if (this.selectedTraitContract) {
      this.router.navigate(['/traits', this.selectedTraitContract.id], { queryParams: { edit: 'true' } });
    } else {
      this.router.navigate(['/traits'], { queryParams: { edit: 'true' } });
    }
  }

  cancelEdit(): void {
    if (this.selectedTraitContract) {
      this.selectTraitContract(this.selectedTraitContract, false);
    } else {
      this.isEditing = false;
      this.router.navigate(['/traits'], { queryParams: {} });
    }
  }

  confirmDeleteState(): void {
    this.showDeleteConfirm = true;
  }

  cancelDelete(): void {
    this.showDeleteConfirm = false;
  }

  saveTraitContract(): void {
    if (!this.traitForm.name) return;

    if (this.selectedTraitContract?.id && this.selectedTraitContract.id.length > 10 && !this.selectedTraitContract.id.startsWith('trait-')) {
      // Update existing trait on backend
      this.apiService.updateTrait(this.selectedTraitContract.id, this.traitForm).subscribe({
        next: (updated) => {
          const idx = this.traitContracts.findIndex(t => t.id === updated.id);
          if (idx >= 0) {
            this.traitContracts[idx] = updated;
          }
          this.snackBar.open(`Updated trait ${updated.name} (v${updated.version})`, 'Close', { duration: 3000 });
          this.selectTraitContract(updated, false);
        },
        error: (err: any) => {
          this.snackBar.open(`Failed to save trait contract: ${err.message || err}`, 'Close', { duration: 3000 });
        }
      });
    } else {
      // Create new trait on backend
      this.apiService.createTrait(this.traitForm).subscribe({
        next: (created) => {
          // Remove local stub if existed
          if (this.selectedTraitContract?.id && this.selectedTraitContract.id.startsWith('trait-')) {
            this.traitContracts = this.traitContracts.filter(t => t.id !== this.selectedTraitContract!.id);
          }
          this.traitContracts.push(created);
          this.snackBar.open(`Created new trait ${created.name}`, 'Close', { duration: 3000 });
          this.selectTraitContract(created, false);
        },
        error: (err: any) => {
          this.snackBar.open(`Failed to create trait contract: ${err.message || err}`, 'Close', { duration: 3000 });
        }
      });
    }
  }

  deleteTraitContract(): void {
    if (!this.selectedTraitContract) return;
    const id = this.selectedTraitContract.id;
    if (id.startsWith('trait-')) {
      // Local stub only
      this.traitContracts = this.traitContracts.filter(t => t.id !== id);
      this.isEditing = false;
      this.showDeleteConfirm = false;
      if (this.traitContracts.length > 0) {
        this.selectTraitContract(this.traitContracts[0]);
      } else {
        this.selectedTraitContract = null;
        this.router.navigate(['/traits'], { queryParams: {} });
      }
      this.snackBar.open('Removed temporary trait definition', 'Close', { duration: 3000 });
      return;
    }

    this.apiService.deleteTrait(id).subscribe({
      next: () => {
        this.traitContracts = this.traitContracts.filter(t => t.id !== id);
        this.isEditing = false;
        this.showDeleteConfirm = false;
        if (this.traitContracts.length > 0) {
          this.selectTraitContract(this.traitContracts[0]);
        } else {
          this.selectedTraitContract = null;
          this.router.navigate(['/traits'], { queryParams: {} });
        }
        this.snackBar.open('Trait contract deleted successfully', 'Close', { duration: 3000 });
      },
      error: (err: any) => {
        this.snackBar.open(`Failed to delete trait contract: ${err.message || err}`, 'Close', { duration: 3000 });
      }
    });
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
