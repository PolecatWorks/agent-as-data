import { Component, OnInit, ViewChild } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';

// Angular Material
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatTooltipModule } from '@angular/material/tooltip';

// Shared Components
import { TopNavbarComponent } from '../../../components/shared/top-navbar/top-navbar.component';
import { EntitySidebarListComponent } from '../../../components/shared/entity-sidebar-list/entity-sidebar-list.component';
import { ConceptGuideComponent, ConceptTabMapping } from '../../../components/concept-guide/concept-guide.component';

import { ApiService } from '../../../services/api.service';

@Component({
  selector: 'app-tool-manager-layout',
  standalone: true,
  imports: [
    CommonModule,
    RouterModule,
    MatIconModule,
    MatButtonModule,
    MatTooltipModule,
    TopNavbarComponent,
    EntitySidebarListComponent,
    ConceptGuideComponent
  ],
  templateUrl: './tool-manager-layout.component.html',
  styleUrl: './tool-manager-layout.component.scss'
})
export class ToolManagerLayoutComponent implements OnInit {
  @ViewChild(TopNavbarComponent) topNavbar?: TopNavbarComponent;
  @ViewChild(ConceptGuideComponent) private _directConceptGuide?: ConceptGuideComponent;

  isSidebarCollapsed = false;
  searchQuery = '';

  mcpServers: any[] = [];

  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'api',
      iconColor: 'text-indigo-600',
      title: '1. Model Context Protocol',
      description: 'Standardized architecture connecting AI agents to external APIs and data sources securely.',
    },
    {
      icon: 'power',
      iconColor: 'text-emerald-600',
      title: '2. Tool Registration',
      description: 'Exposing specific server capabilities as callable JSON-RPC methods for agents.',
    },
    {
      icon: 'security',
      iconColor: 'text-amber-600',
      title: '3. Execution Guardrails',
      description: 'Validating payloads and restricting tool execution to prevent unauthorized system access.',
    },
  ];

  constructor(private apiService: ApiService, private router: Router) {}

  ngOnInit(): void {
    this.loadServers();
  }

  get conceptGuide(): ConceptGuideComponent | undefined {
    return this.topNavbar?.conceptGuide ?? this._directConceptGuide;
  }

  loadServers(): void {
    this.apiService.getTools().subscribe({
      next: (servers) => {
        this.mcpServers = servers; if (this.mcpServers.length > 0 && this.router.url.split('?')[0] === '/tools') { this.router.navigate(['/tools', this.mcpServers[0].id]); }
      },
      error: () => {
        this.mcpServers = [];
      },
    });
  }
  
  get filteredServers(): any[] {
    const q = this.searchQuery.toLowerCase().trim();
    if (!q) return this.mcpServers;
    return this.mcpServers.filter((s) => s.server_name.toLowerCase().includes(q) || (s.description && s.description.toLowerCase().includes(q)));
  }

  toggleSidebar(): void {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }
}
