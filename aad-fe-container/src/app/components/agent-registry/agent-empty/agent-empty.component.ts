import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { EmptyStateComponent } from '../../shared/empty-state/empty-state.component';
import { Router } from '@angular/router';

@Component({
  selector: 'app-agent-empty',
  standalone: true,
  imports: [CommonModule, EmptyStateComponent],
  template: `
    <div class="h-full w-full flex items-center justify-center bg-slate-50">
      <app-empty-state
        icon="smart_toy"
        title="No Agent Selected"
        message="Select an agent from the sidebar to view and edit its configuration, tools, and skills, or create a new one."
        primaryActionLabel="Create New Agent"
        primaryActionIcon="add"
        (primaryAction)="createNewAgent()">
      </app-empty-state>
    </div>
  `,
  styles: []
})
export class AgentEmptyComponent {
  constructor(private router: Router) {}
  
  createNewAgent() {
    this.router.navigate(['/agents'], { queryParams: { create: 'true' } });
  }
}
