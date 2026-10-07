import { Component } from '@angular/core';
import { Router } from '@angular/router';
import { EmptyStateComponent } from '../../shared/empty-state/empty-state.component';

@Component({
  selector: 'app-skill-empty',
  standalone: true,
  imports: [EmptyStateComponent],
  templateUrl: './skill-empty.component.html'
})
export class SkillEmptyComponent {
  constructor(private router: Router) {}

  onCreateNew(): void {
    this.router.navigate(['/skills/new']);
  }
}
