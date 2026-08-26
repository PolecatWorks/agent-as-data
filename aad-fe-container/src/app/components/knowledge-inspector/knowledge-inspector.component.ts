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
    RouterModule
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

  menuItems = [
    { label: 'Home', path: '/home', icon: 'home' },
    { label: 'Agents Registry', path: '/agents', icon: 'smart_toy' },
    { label: 'Traits Registry', path: '/traits', icon: 'verified' },
    { label: 'Skills Registry', path: '/skills', icon: 'extension' },
    { label: 'Tools Registry', path: '/tools', icon: 'dns' },
    { label: 'Testing Studio', path: '/interactive-testing', icon: 'bug_report' },
    { label: 'Workbench', path: '/workbench', icon: 'chat' },
    { label: 'Network Graph', path: '/network-visualizer', icon: 'account_tree' },
    { label: 'Refactoring Lab', path: '/refactoring-lab', icon: 'build_circle' },
    { label: 'Knowledge Inspector', path: '/knowledge-inspector', icon: 'library_books' }
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
