import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatSliderModule } from '@angular/material/slider';
import { MatCardModule } from '@angular/material/card';
import { MatChipsModule } from '@angular/material/chips';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';
import { ApiService } from '../../services/api.service';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';
import { ConceptGuideComponent, ConceptTabMapping } from '../concept-guide/concept-guide.component';

@Component({
  selector: 'app-agent-context',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    RouterModule,
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatSliderModule,
    MatCardModule,
    MatChipsModule,
    MatMenuModule,
    ConceptGuideComponent
  ],
  templateUrl: './agent-context.component.html',
  styleUrls: ['./agent-context.component.scss']
})
export class AgentContextComponent {
  menuItems = APP_NAV_MENU_ITEMS;

  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'search',
      iconColor: 'text-indigo-600',
      title: '1. Natural Language Task Context',
      description: 'Searches vector embeddings and keyword indexes to discover relevant agent entities based on high-level task goals.'
    },
    {
      icon: 'tune',
      iconColor: 'text-emerald-600',
      title: '2. AST Trace Depth',
      description: 'Configures structural subgraph traversal depth across agent skills, traits, and prompt boundaries.'
    },
    {
      icon: 'psychology',
      iconColor: 'text-blue-600',
      title: '3. Top Semantic Matches',
      description: 'Ranks and previews matched agent components with entity type indicators and relevance scoring.'
    }
  ];

  searchQuery: string = '';
  traceDepth: number = 5;
  searchResults: any[] = [];
  isSearching: boolean = false;

  constructor(private apiService: ApiService) {}

  onSearch(): void {
    if (!this.searchQuery.trim()) return;
    this.isSearching = true;
    this.apiService.searchAgentContext(this.searchQuery, this.traceDepth).subscribe({
      next: (results) => {
        this.searchResults = results;
        this.isSearching = false;
      },
      error: (err) => {
        console.error('Error searching context:', err);
        this.isSearching = false;
      }
    });
  }
}
