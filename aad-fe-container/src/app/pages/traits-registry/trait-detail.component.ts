import {
  Component,
  OnInit,
  ChangeDetectorRef,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';

// Angular Material
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSelectModule } from '@angular/material/select';
import { MatTabsModule } from '@angular/material/tabs';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatMenuModule } from '@angular/material/menu';

import { ApiService } from '../../services/api.service';
import { TraitContract } from '../../services/api.service';
import { GuardrailsEditorComponent } from '../../components/guardrails-editor/guardrails-editor.component';

@Component({
  selector: 'app-trait-detail',
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
    RouterModule,
  ],
  templateUrl: './trait-detail.component.html',
  styleUrl: './trait-detail.component.scss',
})
export class TraitDetailComponent implements OnInit {
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
      output_guardrails: { active_guardrails: [] },
    },
  };

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
    private router: Router,
    private cdr: ChangeDetectorRef,
  ) {}

  ngOnInit(): void {
    this.route.paramMap.subscribe(params => {
      const id = params.get('id');
      if (this.route.snapshot.routeConfig?.path === "new") {
        this.createNewTraitContract();
      } else if (id) {
        this.selectTraitById(id);
      }
    });
    this.route.queryParams.subscribe((queryParams) => {
      if (this.route.snapshot.routeConfig?.path !== "new") {
        this.isEditing = queryParams['edit'] === 'true';
      }
    });
  }

  selectTraitById(id: string): void {
    this.apiService.getTrait(id).subscribe({
      next: (trait: TraitContract) => {
        this.selectTraitContract(trait, this.isEditing);
      },
      error: () => {
        this.snackBar.open(`Failed to load trait ${id}`, 'Close', { duration: 3000 });
      }
    });
  }

  selectTraitContract(trait: TraitContract, keepEdit = false): void {
    this.selectedTraitContract = trait;
    this.traitForm = JSON.parse(JSON.stringify(trait));
    this.newRequirement = '';
    this.newInvariant = '';
    this.newCriterion = '';
    this.newTag = '';
    this.showDeleteConfirm = false;
    
    if (!keepEdit) {
      this.isEditing = false;
      this.router.navigate(['/traits', trait.id]);
    } else {
      this.router.navigate(['/traits', trait.id], { queryParams: { edit: 'true' } });
    }
  }

  createNewTraitContract(): void {
    const newTrait: TraitContract = {
      owner_id: '00000000-0000-0000-0000-000000000000',
      id: `trait-${Date.now()}`,
      name: 'New Custom Trait',
      description: '',
      version: '1.0.0',
      capability_requirements: [],
      behavioral_invariants: [],
      evaluation_criteria: [],
      tags: ['custom'],
      guardrails: {
        input_guardrails: { active_guardrails: [] },
        output_guardrails: { active_guardrails: [] },
      },
    };
    
    this.selectedTraitContract = newTrait;
    this.traitForm = JSON.parse(JSON.stringify(newTrait));
    this.isEditing = true;
    this.showDeleteConfirm = false;
  }

  enableEdit(): void {
    if (!this.selectedTraitContract) return;
    this.isEditing = true;
    this.traitForm = JSON.parse(JSON.stringify(this.selectedTraitContract));
    this.router.navigate(['/traits', this.selectedTraitContract.id], {
      queryParams: { edit: 'true' },
    });
  }

  cancelEdit(): void {
    if (!this.selectedTraitContract) return;
    if (this.selectedTraitContract.id.startsWith('trait-')) {
      this.router.navigate(['/traits']);
      return;
    }
    
    this.isEditing = false;
    this.traitForm = JSON.parse(JSON.stringify(this.selectedTraitContract));
    this.router.navigate(['/traits', this.selectedTraitContract.id]);
  }

  confirmDeleteState(): void {
    this.showDeleteConfirm = true;
  }

  cancelDelete(): void {
    this.showDeleteConfirm = false;
  }

  saveTraitContract(): void {
    if (!this.traitForm.name) return;

    if (
      this.selectedTraitContract?.id &&
      this.selectedTraitContract.id.length > 10 &&
      !this.selectedTraitContract.id.startsWith('trait-')
    ) {
      this.apiService.updateTrait(this.selectedTraitContract.id, this.traitForm).subscribe({
        next: (updated) => {
          this.snackBar.open(`Updated trait ${updated.name} (v${updated.version})`, 'Close', { duration: 3000 });
          this.selectTraitContract(updated, false);
        },
        error: (err: any) => {
          this.snackBar.open(`Failed to save trait contract: ${err.message || err}`, 'Close', { duration: 3000 });
        },
      });
    } else {
      this.apiService.createTrait(this.traitForm).subscribe({
        next: (created) => {
          this.snackBar.open(`Created new trait ${created.name}`, 'Close', { duration: 3000 });
          this.selectTraitContract(created, false);
        },
        error: (err: any) => {
          this.snackBar.open(`Failed to create trait contract: ${err.message || err}`, 'Close', { duration: 3000 });
        },
      });
    }
  }

  deleteTraitContract(): void {
    if (!this.selectedTraitContract) return;
    const id = this.selectedTraitContract.id;
    if (id.startsWith('trait-')) {
      this.isEditing = false;
      this.showDeleteConfirm = false;
      this.selectedTraitContract = null;
      this.router.navigate(['/traits']);
      this.snackBar.open('Removed temporary trait definition', 'Close', { duration: 3000 });
      return;
    }

    this.apiService.deleteTrait(id).subscribe({
      next: () => {
        this.isEditing = false;
        this.showDeleteConfirm = false;
        this.selectedTraitContract = null;
        this.router.navigate(['/traits']);
        this.snackBar.open('Trait contract deleted successfully', 'Close', { duration: 3000 });
      },
      error: (err: any) => {
        this.snackBar.open(`Failed to delete: ${err.message || err}`, 'Close', { duration: 3000 });
      },
    });
  }

  // Capability Requirements Helper Methods
  addRequirement(): void {
    if (this.newRequirement.trim()) {
      if (!this.traitForm.capability_requirements)
        this.traitForm.capability_requirements = [];
      this.traitForm.capability_requirements.push(this.newRequirement.trim());
      this.newRequirement = '';
    }
  }

  removeRequirement(req: string): void {
    if (this.traitForm.capability_requirements) {
      this.traitForm.capability_requirements =
        this.traitForm.capability_requirements.filter((r) => r !== req);
    }
  }

  // Behavioral Invariants Helper Methods
  addInvariant(): void {
    if (this.newInvariant.trim()) {
      if (!this.traitForm.behavioral_invariants)
        this.traitForm.behavioral_invariants = [];
      this.traitForm.behavioral_invariants.push(this.newInvariant.trim());
      this.newInvariant = '';
    }
  }

  removeInvariant(inv: string): void {
    if (this.traitForm.behavioral_invariants) {
      this.traitForm.behavioral_invariants =
        this.traitForm.behavioral_invariants.filter((i) => i !== inv);
    }
  }

  // Evaluation Criteria Helper Methods
  addCriterion(): void {
    if (this.newCriterion.trim()) {
      if (!this.traitForm.evaluation_criteria)
        this.traitForm.evaluation_criteria = [];
      this.traitForm.evaluation_criteria.push(this.newCriterion.trim());
      this.newCriterion = '';
    }
  }

  removeCriterion(crit: string): void {
    if (this.traitForm.evaluation_criteria) {
      this.traitForm.evaluation_criteria =
        this.traitForm.evaluation_criteria.filter((c) => c !== crit);
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
      this.traitForm.tags = this.traitForm.tags.filter((t) => t !== tag);
    }
  }
}
