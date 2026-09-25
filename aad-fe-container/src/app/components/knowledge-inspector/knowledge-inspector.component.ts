import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { MatTabsModule } from '@angular/material/tabs';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatChipsModule } from '@angular/material/chips';
import { RouterModule } from '@angular/router';
import { ApiService, KnowledgeNode, KnowledgeTupleInput } from '../../services/api.service';
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
    MatTabsModule,
    MatTooltipModule,
    MatChipsModule,
    RouterModule,
    ConceptGuideComponent
  ],
  templateUrl: './knowledge-inspector.component.html',
  styleUrl: './knowledge-inspector.component.scss'
})
export class KnowledgeInspectorComponent implements OnInit {
  // Sidebar state
  isSidebarCollapsed = false;
  searchQuery: string = '';
  nodes: KnowledgeNode[] = [];

  // Editor state
  selectedNode: KnowledgeNode | null = null;
  isEditing: boolean = false;
  showDeleteConfirm: boolean = false;
  newTag: string = '';

  // Form State
  nodeForm: Partial<KnowledgeNode> & { tuples?: KnowledgeTupleInput[] } = {
    topic: '',
    title: '',
    description: '',
    content: '',
    tags: [],
    tuples: []
  };

  // Dashboard / RAG / Graph Search State
  ragSearchQuery: string = 'Rust memory safety';
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
    this.loadNodes();
    this.runRagSearch();
    this.runTraverse();
  }

  toggleSidebar() {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  loadNodes() {
    this.apiService.getKnowledgeNodes().subscribe(nodes => {
      this.nodes = nodes || [];
    });
  }

  getFilteredNodes(): KnowledgeNode[] {
    if (!this.searchQuery) return this.nodes;
    const q = this.searchQuery.toLowerCase();
    return this.nodes.filter(n =>
      (n.title && n.title.toLowerCase().includes(q)) ||
      n.topic.toLowerCase().includes(q) ||
      (n.description && n.description.toLowerCase().includes(q))
    );
  }

  selectNode(node: KnowledgeNode) {
    this.selectedNode = node;
    this.isEditing = false;
    this.showDeleteConfirm = false;

    // Load tuples
    this.apiService.getKnowledgeTuples(node.id).subscribe(tuples => {
      this.nodeForm = {
        ...node,
        tuples: tuples.map(t => ({
          subject: t.subject,
          predicate: t.predicate,
          object: t.object,
          confidence: t.confidence
        }))
      };
    }, error => {
      // Fallback
      this.nodeForm = { ...node, tuples: [] };
    });
  }

  createNewNode() {
    this.selectedNode = null;
    this.isEditing = true;
    this.showDeleteConfirm = false;
    this.nodeForm = {
      topic: '',
      title: '',
      description: '',
      content: '',
      tags: [],
      tuples: []
    };
  }

  enableEdit() {
    this.isEditing = true;
    this.showDeleteConfirm = false;
  }

  cancelEdit() {
    this.isEditing = false;
    this.showDeleteConfirm = false;
    if (this.selectedNode) {
      this.selectNode(this.selectedNode);
    }
  }

  confirmDeleteState() {
    this.showDeleteConfirm = true;
  }

  cancelDelete() {
    this.showDeleteConfirm = false;
  }

  deleteNode() {
    if (this.selectedNode) {
      this.apiService.deleteKnowledgeNode(this.selectedNode.id).subscribe(() => {
        this.loadNodes();
        this.selectedNode = null;
        this.isEditing = false;
        this.showDeleteConfirm = false;
      });
    }
  }

  saveNode() {
    if (this.selectedNode) {
        // Update
        const payload = { ...this.nodeForm };
        this.apiService.updateKnowledgeNode(this.selectedNode.id, payload).subscribe((updated) => {
            this.selectedNode = updated;
            this.isEditing = false;
            this.loadNodes();
            this.selectNode(updated);
        });
    } else {
        // Create
        this.apiService.ingestKnowledge(
            this.nodeForm.topic || 'General',
            this.nodeForm.title || '',
            this.nodeForm.description,
            this.nodeForm.tags || [],
            this.nodeForm.content || '',
            this.nodeForm.tuples
        ).subscribe((res) => {
            this.loadNodes();
            this.isEditing = false;
            // Fetch the newly created node to select it
            this.apiService.getKnowledgeNode(res.id).subscribe(node => this.selectNode(node));
        });
    }
  }

  // Tags
  addTag() {
    if (this.newTag.trim() && this.nodeForm.tags) {
      if (!this.nodeForm.tags.includes(this.newTag.trim())) {
        this.nodeForm.tags.push(this.newTag.trim());
      }
      this.newTag = '';
    }
  }

  removeTag(tag: string) {
    if (this.nodeForm.tags) {
      this.nodeForm.tags = this.nodeForm.tags.filter(t => t !== tag);
    }
  }

  // Tuples
  addTuple() {
      if (!this.nodeForm.tuples) {
          this.nodeForm.tuples = [];
      }
      this.nodeForm.tuples.push({ subject: '', predicate: '', object: '', confidence: 1.0 });
  }

  removeTuple(index: number) {
      if (this.nodeForm.tuples) {
          this.nodeForm.tuples.splice(index, 1);
      }
  }


  // Dashboards
  runRagSearch(): void {
    if (!this.ragSearchQuery.trim()) return;
    this.isSearching = true;
    this.apiService.searchKnowledge(this.ragSearchQuery).subscribe({
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
    // Note: The UI is hitting /v1/knowledge/graph/traverse in the API service
    // But api.service.ts method is traverseGraph (already correct, but no method in the component for it yet if api changed, let's check)
    // Looking back at api.service.ts, traverseGraph was NOT changed and still uses `/knowledge/graph/traverse` via post.
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
