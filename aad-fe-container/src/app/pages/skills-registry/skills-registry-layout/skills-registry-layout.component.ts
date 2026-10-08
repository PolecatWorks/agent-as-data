import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterModule } from '@angular/router';
import { TopNavbarComponent } from '../../../components/shared/top-navbar/top-navbar.component';
import { EntitySidebarListComponent } from '../../../components/shared/entity-sidebar-list/entity-sidebar-list.component';
import { ApiService, Skill } from '../../../services/api.service';
import { ConceptTabMapping } from '../../../components/concept-guide/concept-guide.component';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatTooltipModule } from '@angular/material/tooltip';

@Component({
  selector: 'app-skills-registry-layout',
  standalone: true,
  imports: [CommonModule, RouterModule, TopNavbarComponent, EntitySidebarListComponent, MatIconModule, MatButtonModule, MatTooltipModule],
  templateUrl: './skills-registry-layout.component.html'
})
export class SkillsRegistryLayoutComponent implements OnInit {
  skills: Skill[] = [];
  isSidebarCollapsed = false;
  searchQuery = '';
  
  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'description',
      iconColor: 'text-indigo-600',
      title: '1. Procedural Instructions',
      description: 'Step-by-step guidance and deterministic execution rules defining how the task is performed.',
    },
    {
      icon: 'schema',
      iconColor: 'text-amber-600',
      title: '2. Typed JSON Schemas',
      description: 'Strictly validated input parameters and structured response payload schemas.',
    },
    {
      icon: 'verified',
      iconColor: 'text-emerald-600',
      title: '3. Trait Safety Verification',
      description: 'Automated contract verification ensuring the skill adheres to required behavioral invariants.',
    },
  ];

  constructor(
    private apiService: ApiService,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadSkills();
  }

  loadSkills(): void {
    this.apiService.getSkills().subscribe({
      next: (skills) => {
        this.skills = skills; if (this.skills.length > 0 && this.router.url.split('?')[0] === '/skills') { this.router.navigate(['/skills', this.skills[0].id]); }
      },
      error: (err) => {
        console.error('Failed to load skills', err);
      }
    });
  }

  getFilteredSkills(): Skill[] {
    const query = this.searchQuery.toLowerCase().trim();
    if (!query) return this.skills;
    return this.skills.filter(s =>
      s.name.toLowerCase().includes(query) ||
      s.description.toLowerCase().includes(query) ||
      (s.tags || []).some((t: string) => t.toLowerCase().includes(query))
    );
  }

  onSelectSkill(skill: any): void {
    // Selection state is managed by routerLinkActive in the template
  }

  createNewSkill(): void {
    this.router.navigate(['/skills/new']);
  }
}
