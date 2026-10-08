import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterModule } from '@angular/router';
import { TopNavbarComponent } from '../../../components/shared/top-navbar/top-navbar.component';
import { EntitySidebarListComponent } from '../../../components/shared/entity-sidebar-list/entity-sidebar-list.component';
import { ApiService, TraitContract } from '../../../services/api.service';
import { ConceptTabMapping } from '../../../components/concept-guide/concept-guide.component';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatTooltipModule } from '@angular/material/tooltip';

@Component({
  selector: 'app-traits-registry-layout',
  standalone: true,
  imports: [CommonModule, RouterModule, TopNavbarComponent, EntitySidebarListComponent, MatIconModule, MatButtonModule, MatTooltipModule],
  templateUrl: './traits-registry-layout.component.html'
})
export class TraitsRegistryLayoutComponent implements OnInit {
  traits: TraitContract[] = [];
  isSidebarCollapsed = false;
  searchQuery = '';
  
  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'verified',
      iconColor: 'text-emerald-600',
      title: '1. Guardrails & Restrictions',
      description: 'Strict behavioral limits to prevent harmful or unintended agent actions.',
    },
    {
      icon: 'description',
      iconColor: 'text-indigo-600',
      title: '2. Required Formatting',
      description: 'Rules for how the agent must structure and format its outputs.',
    },
    {
      icon: 'gavel',
      iconColor: 'text-amber-600',
      title: '3. Compliance Policies',
      description: 'Domain-specific regulatory and corporate policies the agent must follow.',
    },
  ];

  constructor(
    private apiService: ApiService,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadTraits();
  }

  loadTraits(): void {
    this.apiService.getTraits().subscribe({
      next: (listRes: any) => {
        const ids = listRes.ids || [];
        if (ids.length > 0) {
          // In a real app we'd fetch bulk or use a summary endpoint, 
          // but for the layout we need names and descriptions.
          // Since getTraits just returns IDs, we'll fetch them individually.
          this.traits = ids.map((id: string) => ({ name: id, description: 'Loading...', guardrails: [], requirements: [] }));
          
          ids.forEach((id: string, index: number) => {
             this.apiService.getTrait(id).subscribe({
                next: (fullTrait: TraitContract) => {
                   this.traits[index] = fullTrait;
                   this.traits = [...this.traits];
                }
             });
          });
        }
      },
      error: (err) => {
        console.error('Failed to load traits', err);
      }
    });
  }

  getFilteredTraits(): TraitContract[] {
    const query = this.searchQuery.toLowerCase().trim();
    if (!query) return this.traits;
    return this.traits.filter(t =>
      t.name.toLowerCase().includes(query) ||
      (t.description && t.description.toLowerCase().includes(query))
    );
  }

  onSelectTrait(trait: any): void {
    // Selection state is managed by routerLinkActive in the template
  }

  createNewTrait(): void {
    this.router.navigate(['/traits/new']);
  }
}
