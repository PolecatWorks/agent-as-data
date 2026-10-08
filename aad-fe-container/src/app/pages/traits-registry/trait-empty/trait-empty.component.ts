import { Component } from '@angular/core';
import { Router } from '@angular/router';
import { EmptyStateComponent } from '../../../components/shared/empty-state/empty-state.component';

@Component({
  selector: 'app-trait-empty',
  standalone: true,
  imports: [EmptyStateComponent],
  template: `
    <div class="h-full flex items-center justify-center p-8">
      <app-empty-state
        icon="verified"
        title="No Trait Selected"
        message="Select a trait from the sidebar to view its behavioral contracts, or create a new one to define new guardrails."
        actionLabel="Create New Trait"
        (actionClick)="onCreateNew()">
      </app-empty-state>
    </div>
  `
})
export class TraitEmptyComponent {
  constructor(private router: Router) {}

  onCreateNew(): void {
    this.router.navigate(['/traits/new']);
  }
}
