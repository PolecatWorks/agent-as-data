import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatChipsModule } from '@angular/material/chips';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { RouterModule } from '@angular/router';
import { ApiService, KnowledgeNode } from '../../services/api.service';
import { ConceptGuideComponent, ConceptTabMapping } from '../concept-guide/concept-guide.component';
import { APP_NAV_MENU_ITEMS } from '../../models/navigation';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import { forkJoin } from 'rxjs';

export interface KnowledgeNodeProposal {
  topic: string;
  title: string;
  description: string;
  tags: string[];
  content: string;
  selected?: boolean;
}

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
    MatTooltipModule,
    MatChipsModule,
    MatProgressSpinnerModule,
    MatCheckboxModule,
    RouterModule,
    ConceptGuideComponent
  ],
  templateUrl: './knowledge-inspector.component.html',
  styleUrl: './knowledge-inspector.component.scss'
})
export class KnowledgeInspectorComponent implements OnInit {
  isSidebarCollapsed: boolean = false;
  nodes: KnowledgeNode[] = [];
  selectedNode: KnowledgeNode | null = null;
  searchQuery: string = '';

  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;
  nodeForm: Partial<KnowledgeNode> = {};
  newTag: string = '';

  // Markdown Import state
  showMarkdownImport: boolean = false;
  markdownInput: string = '';
  isAnalyzing: boolean = false;
  importProposals: KnowledgeNodeProposal[] = [];

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
    this.loadNodes();
  }

  loadNodes(): void {
    this.apiService.getKnowledgeNodes().subscribe({
      next: (nodes) => {
        this.nodes = nodes;
      },
      error: (err) => {
        console.error('Failed to load knowledge nodes', err);
      }
    });
  }

  getFilteredNodes(): KnowledgeNode[] {
    if (!this.searchQuery.trim()) return this.nodes;
    const q = this.searchQuery.toLowerCase();
    return this.nodes.filter(n =>
      (n.topic && n.topic.toLowerCase().includes(q)) ||
      (n.title && n.title.toLowerCase().includes(q)) ||
      (n.description && n.description.toLowerCase().includes(q))
    );
  }

  selectNode(node: KnowledgeNode): void {
    this.selectedNode = node;
    this.isEditing = false;
    this.showDeleteConfirm = false;
    this.showMarkdownImport = false;
    this.nodeForm = JSON.parse(JSON.stringify(node));
  }

  createNewNode(): void {
    this.selectedNode = null;
    this.isEditing = true;
    this.showDeleteConfirm = false;
    this.showMarkdownImport = false;
    this.nodeForm = {
      topic: 'general',
      title: '',
      description: '',
      tags: [],
      content: ''
    };
  }

  openMarkdownImport(): void {
    this.selectedNode = null;
    this.isEditing = false;
    this.showDeleteConfirm = false;
    this.showMarkdownImport = true;
    this.markdownInput = '';
    this.importProposals = [];
  }

  cancelMarkdownImport(): void {
    this.showMarkdownImport = false;
    this.markdownInput = '';
    this.importProposals = [];
  }

  analyzeMarkdown(): void {
    if (!this.markdownInput.trim()) return;

    this.isAnalyzing = true;
    this.importProposals = [];

    this.apiService.analyzeMarkdown(this.markdownInput).subscribe({
      next: (res) => {
        this.isAnalyzing = false;
        this.importProposals = res.proposals.map(p => ({ ...p, selected: true }));
      },
      error: (err) => {
        this.isAnalyzing = false;
        console.error('Failed to analyze markdown', err);
      }
    });
  }

  finalizeImport(): void {
    const selectedProposals = this.importProposals.filter(p => p.selected);
    if (selectedProposals.length === 0) return;

    const requests = selectedProposals.map(p =>
      this.apiService.ingestKnowledge(p.topic, p.title, p.description, p.tags, p.content)
    );

    forkJoin(requests).subscribe({
      next: () => {
        this.cancelMarkdownImport();
        this.loadNodes();
      },
      error: (err) => {
        console.error('Failed to finalize import', err);
      }
    });
  }

  toggleSidebar(): void {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  enableEdit(): void {
    this.isEditing = true;
  }

  cancelEdit(): void {
    this.isEditing = false;
    if (this.selectedNode) {
      this.nodeForm = JSON.parse(JSON.stringify(this.selectedNode));
    } else {
      this.nodeForm = {};
    }
  }

  confirmDeleteState(): void {
    this.showDeleteConfirm = true;
  }

  cancelDelete(): void {
    this.showDeleteConfirm = false;
  }

  deleteNode(): void {
    if (!this.selectedNode?.id) return;
    this.apiService.deleteKnowledgeNode(this.selectedNode.id).subscribe({
      next: () => {
        this.showDeleteConfirm = false;
        this.selectedNode = null;
        this.isEditing = false;
        this.nodeForm = {};
        this.loadNodes();
      },
      error: (err) => {
        console.error('Failed to delete node', err);
      }
    });
  }

  saveNode(): void {
    if (this.selectedNode?.id) {
      this.apiService.updateKnowledgeNode(this.selectedNode.id, this.nodeForm).subscribe({
        next: (node) => {
          this.selectedNode = node;
          this.isEditing = false;
          this.loadNodes();
        },
        error: (err) => console.error('Failed to update node', err)
      });
    } else {
      this.apiService.ingestKnowledge(
        this.nodeForm.topic || 'general',
        this.nodeForm.title || '',
        this.nodeForm.description,
        this.nodeForm.tags || [],
        this.nodeForm.content || ''
      ).subscribe({
        next: (res) => {
          this.isEditing = false;
          this.loadNodes();
          // We don't have the full node back from ingest usually, so we'll just reload
          if (res && res.id) {
            // we could pre-select it
          }
        },
        error: (err) => console.error('Failed to create node', err)
      });
    }
  }

  addTag(): void {
    if (this.newTag.trim() !== '') {
      if (!this.nodeForm.tags) {
        this.nodeForm.tags = [];
      }
      if (!this.nodeForm.tags.includes(this.newTag.trim())) {
        this.nodeForm.tags.push(this.newTag.trim());
      }
      this.newTag = '';
    }
  }

  removeTag(tag: string): void {
    if (this.nodeForm.tags) {
      this.nodeForm.tags = this.nodeForm.tags.filter(t => t !== tag);
    }
  }

  getRenderedMarkdown(content: string): string {
    if (!content) return '';
    try {
      return DOMPurify.sanitize(marked.parse(content) as string);
    } catch {
      return content;
    }
  }
}
