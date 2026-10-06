import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { TopNavbarComponent } from '../../shared/top-navbar/top-navbar.component';
import { EntitySidebarListComponent } from '../../shared/entity-sidebar-list/entity-sidebar-list.component';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule,
    } from '@angular/material/button';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatSnackBar } from '@angular/material/snack-bar';
import { ApiService, Agent } from '../../../services/api.service';
import { ConceptTabMapping } from '../../concept-guide/concept-guide.component';

@Component({
  selector: 'app-agent-registry-layout',
  standalone: true,
  imports: [
    CommonModule, 
    RouterModule, 
    TopNavbarComponent, 
    EntitySidebarListComponent,
    MatIconModule,
    MatButtonModule,
    MatTooltipModule
  ],
  templateUrl: './agent-registry-layout.component.html',
  styleUrl: './agent-registry-layout.component.scss'
})
export class AgentRegistryLayoutComponent implements OnInit {
  agents: Agent[] = [];
  searchQuery: string = '';
  isSidebarCollapsed: boolean = false;

  conceptGuideMappings: ConceptTabMapping[] = [
    { title: 'Overview', icon: 'visibility', description: 'Agents act as autonomous nodes that reason through context, utilize Tools, execute Skills, and interface with Knowledge to fulfill complex tasks.', iconColor: 'text-indigo-500' },
    { title: 'Configuration', icon: 'settings', description: 'Agents are declarative configurations defined by a System Prompt, Temperature, attached Skills, Tools, and Trait Contracts.', iconColor: 'text-indigo-500' },
    { title: 'Trait Compliance', icon: 'shield_with_heart', description: 'Agents implement Trait Contracts to guarantee they possess specific operational signatures before they are assigned to multi-turn workflows.', iconColor: 'text-indigo-500' }
  ];

  constructor(
    private apiService: ApiService,
    private router: Router,
    private snackBar: MatSnackBar
  ) {}

  ngOnInit() {
    this.loadAgents();
  }

  loadAgents(): void {
    this.apiService.getAgents().subscribe({
      next: (list) => {
        this.agents = list;
      },
      error: (err) => {
        this.snackBar.open(`Error loading agents: ${err.message || err}`, 'Close', { duration: 5000 });
      }
    });
  }

  getFilteredAgents(): Agent[] {
    const q = this.searchQuery.toLowerCase().trim();
    if (!q) return this.agents;
    return this.agents.filter(a => a.name.toLowerCase().includes(q) || (a.description && a.description.toLowerCase().includes(q)));
  }

  createNewAgent() {
    this.router.navigate(['/agents/new']);
  }

  onSelectAgent(agent: Agent) {
    this.router.navigate(['/agents', agent.id]);
  }
}
