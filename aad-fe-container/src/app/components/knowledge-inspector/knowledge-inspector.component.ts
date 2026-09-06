import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { RouterModule } from '@angular/router';
import { ApiService } from '../../services/api.service';
import { ConceptGuideComponent, ConceptTabMapping } from '../concept-guide/concept-guide.component';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';

@Component({
  selector: 'app-knowledge-inspector',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatIconModule,
    MatMenuModule,
    RouterModule,
    ConceptGuideComponent
  ],
  templateUrl: './knowledge-inspector.component.html',
  styleUrl: './knowledge-inspector.component.scss'
})
export class KnowledgeInspectorComponent implements OnInit {
  searchQuery: string = 'Rust memory safety';
  searchResults: any[] = [];
  subjectQuery: string = 'SecurityAuditor';
  graphResults: any[] = [];
  isSearching: boolean = false;

  menuItems = APP_NAV_MENU_ITEMS;

  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'saved_search',
      iconColor: 'text-blue-600',
      title: '1. Semantic Vector Store',
      description: 'High-dimensional vector embeddings for hybrid RAG search over documents and corporate policies.'
    },
    {
      icon: 'hub',
      iconColor: 'text-indigo-600',
      title: '2. Knowledge Graph Triples',
      description: 'Subject-Predicate-Object relation tuples connecting company concepts, teams, and data structures.'
    },
    {
      icon: 'cleaning_services',
      iconColor: 'text-emerald-600',
      title: '3. Entity Resolution & Pruning',
      description: 'Canonical entity deduction and automated duplicate pruning ensuring reliable AI grounding.'
    }
  ];

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.runSearch();
    this.runTraverse();
  }

  runSearch(): void {
    if (!this.searchQuery.trim()) return;
    this.isSearching = true;
    this.apiService.searchKnowledge(this.searchQuery).subscribe({
      next: (res) => {
        this.isSearching = false;
        this.searchResults = res || [];
      },
      error: () => {
        this.isSearching = false;
        this.searchResults = [
          {
            chunk_index: 0,
            chunk_text: 'Rust enforces memory safety via ownership, borrowing, and lifetime rules without requiring garbage collection.',
            score: 0.94
          }
        ];
      }
    });
  }

  runTraverse(): void {
    if (!this.subjectQuery.trim()) return;
    this.apiService.traverseGraph(this.subjectQuery).subscribe({
      next: (res) => {
        this.graphResults = res || [];
      },
      error: () => {
        this.graphResults = [
          { subject: 'SecurityAuditor', predicate: 'implements', object: 'SecurityTrait', confidence: 1.0 },
          { subject: 'SecurityAuditor', predicate: 'uses_tool', object: 'RustMemoryScan', confidence: 0.95 }
        ];
      }
    });
  }
}
