import { Component, OnInit } from '@angular/core';
import { TopNavbarComponent } from '../../components/shared/top-navbar/top-navbar.component';
import { EntitySidebarListComponent } from '../../components/shared/entity-sidebar-list/entity-sidebar-list.component';


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
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { ActivatedRoute, Router, RouterModule } from '@angular/router';
import {
  ApiService,
  KnowledgeNode,
  KnowledgeTupleInput,
} from '../../services/api.service';
import {
  ConceptGuideComponent,
  ConceptTabMapping,
} from '../../components/concept-guide/concept-guide.component';
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
    EntitySidebarListComponent,
    TopNavbarComponent,
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
    MatProgressSpinnerModule,
    MatCheckboxModule,
    RouterModule,
    ConceptGuideComponent,
  ],
  templateUrl: './knowledge-inspector.component.html',
  styleUrl: './knowledge-inspector.component.scss',
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
    tuples: [],
  };

  // Dashboard / RAG / Graph Search State
  ragSearchQuery: string = 'Rust memory safety';
  searchResults: any[] = [];
  subjectQuery: string = 'SecurityAuditor';
  graphResults: any[] = [];
  isSearching: boolean = false;

  // Markdown Import state
  showMarkdownImport: boolean = false;
  markdownInput: string = '';
  isAnalyzing: boolean = false;
  importDocumentProposal: KnowledgeNodeProposal | null = null;
  saveSourceDocument: boolean = true;
  importProposals: (KnowledgeNodeProposal & { selected?: boolean })[] = [];

  // Derived concepts state
  derivedConcepts: KnowledgeNode[] = [];
  isLoadingDerivedConcepts: boolean = false;

  menuItems = APP_NAV_MENU_ITEMS;

  readonly conceptGuideMappings: ConceptTabMapping[] = [
    {
      icon: 'saved_search',
      iconColor: 'text-blue-600',
      title: '1. Semantic Vector Store',
      description:
        'High-dimensional vector embeddings for hybrid RAG search over documents and corporate policies.',
    },
    {
      icon: 'hub',
      iconColor: 'text-indigo-600',
      title: '2. Knowledge Graph Triples',
      description:
        'Subject-Predicate-Object relation tuples connecting company concepts, teams, and data structures.',
    },
    {
      icon: 'cleaning_services',
      iconColor: 'text-emerald-600',
      title: '3. Entity Resolution & Pruning',
      description:
        'Canonical entity deduction and automated duplicate pruning ensuring reliable AI grounding.',
    },
  ];

  constructor(
    private apiService: ApiService,
    private route: ActivatedRoute,
    private router: Router,
  ) {}

  ngOnInit(): void {
    this.loadNodes();
    this.runRagSearch();
    this.runTraverse();

    this.route.paramMap.subscribe((params) => {
      const id = params.get('id');
      if (id) {
        this.handleRouteSelection(id);
      } else if (!this.isEditing && !this.showMarkdownImport) {
        this.selectedNode = null;
      }
    });
  }

  toggleSidebar() {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  loadNodes() {
    this.apiService.getKnowledgeNodes().subscribe((nodes) => {
      this.nodes = nodes || [];
      const routeId = this.route.snapshot.paramMap.get('id');
      if (!routeId && this.nodes.length > 0 && this.router.url.split('?')[0] === '/knowledge-inspector') { this.router.navigate(['/knowledge-inspector', this.nodes[0].id]); } else if (routeId && (!this.selectedNode || this.selectedNode.id !== routeId)) {
        this.handleRouteSelection(routeId);
      }
    });
  }

  handleRouteSelection(id: string): void {
    const existing = this.nodes.find((n) => n.id === id);
    if (existing) {
      this.selectNode(existing, false);
    } else {
      this.apiService.getKnowledgeNode(id).subscribe({
        next: (node) => {
          this.selectNode(node, false);
        },
        error: (err) => {
          console.error('Failed to load knowledge node from route id', err);
        },
      });
    }
  }

  getFilteredNodes(): KnowledgeNode[] {
    if (!this.searchQuery) return this.nodes;
    const q = this.searchQuery.toLowerCase();
    return this.nodes.filter(
      (n) =>
        (n.title && n.title.toLowerCase().includes(q)) ||
        n.topic.toLowerCase().includes(q) ||
        (n.description && n.description.toLowerCase().includes(q)),
    );
  }

  selectNode(node: KnowledgeNode, triggerNavigation: boolean = true) {
    this.selectedNode = node;
    this.isEditing = false;
    this.showDeleteConfirm = false;
    this.showMarkdownImport = false;
    this.derivedConcepts = [];

    if (triggerNavigation) {
      this.router.navigate(['/knowledge-inspector', node.id]);
    }

    // Load tuples
    this.apiService.getKnowledgeTuples(node.id).subscribe(
      (tuples) => {
        this.nodeForm = {
          ...node,
          tuples: tuples.map((t) => ({
            subject: t.subject,
            predicate: t.predicate,
            object: t.object,
            confidence: t.confidence,
          })),
        };
      },
      (error) => {
        // Fallback
        this.nodeForm = { ...node, tuples: [] };
      },
    );

    // Load derived concepts if this is a source document
    if (node.metadata?.is_source_document) {
      this.isLoadingDerivedConcepts = true;
      this.apiService.getDerivedConcepts(node.id).subscribe({
        next: (concepts) => {
          this.derivedConcepts = concepts;
          this.isLoadingDerivedConcepts = false;
        },
        error: () => {
          this.derivedConcepts = [];
          this.isLoadingDerivedConcepts = false;
        },
      });
    }
  }

  navigateToNodeById(nodeId: string): void {
    if (!nodeId) return;
    this.router.navigate(['/knowledge-inspector', nodeId]);
  }

  createNewNode() {
    this.selectedNode = null;
    this.isEditing = true;
    this.showDeleteConfirm = false;
    this.showMarkdownImport = false;
    this.derivedConcepts = [];
    this.router.navigate(['/knowledge-inspector']);
    this.nodeForm = {
      topic: '',
      title: '',
      description: '',
      content: '',
      tags: [],
      tuples: [],
    };
  }

  openMarkdownImport(): void {
    this.selectedNode = null;
    this.isEditing = false;
    this.showDeleteConfirm = false;
    this.showMarkdownImport = true;
    this.markdownInput = '';
    this.importDocumentProposal = null;
    this.saveSourceDocument = true;
    this.importProposals = [];
    this.router.navigate(['/knowledge-inspector']);
  }

  cancelMarkdownImport(): void {
    this.showMarkdownImport = false;
    this.markdownInput = '';
    this.importDocumentProposal = null;
    this.importProposals = [];
  }

  analyzeMarkdown(): void {
    if (!this.markdownInput.trim()) return;

    this.isAnalyzing = true;
    this.importProposals = [];
    this.importDocumentProposal = null;

    this.apiService.analyzeMarkdown(this.markdownInput).subscribe({
      next: (res) => {
        this.isAnalyzing = false;
        this.importDocumentProposal = res.document || null;
        this.importProposals = (res.proposals || []).map((p: any) => ({ ...p, selected: true }));
      },
      error: (err) => {
        this.isAnalyzing = false;
        console.error('Failed to analyze markdown', err);
      },
    });
  }

  finalizeImport(): void {
    const selectedProposals = this.importProposals.filter((p) => p.selected);
    if (!this.importDocumentProposal && selectedProposals.length === 0) return;

    if (this.saveSourceDocument && this.importDocumentProposal) {
      this.apiService.importDocument(this.importDocumentProposal, selectedProposals, true).subscribe({
        next: () => {
          this.cancelMarkdownImport();
          this.loadNodes();
        },
        error: (err) => {
          console.error('Failed to finalize document import', err);
        },
      });
    } else {
      const requests = selectedProposals.map((p) =>
        this.apiService.ingestKnowledge(p.topic, p.title, p.description, p.tags, p.content),
      );

      forkJoin(requests).subscribe({
        next: () => {
          this.cancelMarkdownImport();
          this.loadNodes();
        },
        error: (err) => {
          console.error('Failed to finalize import', err);
        },
      });
    }
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
    } else {
      this.router.navigate(['/knowledge-inspector']);
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
      this.apiService
        .deleteKnowledgeNode(this.selectedNode.id)
        .subscribe(() => {
          this.loadNodes();
          this.selectedNode = null;
          this.isEditing = false;
          this.showDeleteConfirm = false;
          this.router.navigate(['/knowledge-inspector']);
        });
    }
  }

  saveNode() {
    if (this.selectedNode) {
      // Update
      const payload = { ...this.nodeForm };
      this.apiService
        .updateKnowledgeNode(this.selectedNode.id, payload)
        .subscribe((updated) => {
          this.selectedNode = updated;
          this.isEditing = false;
          this.loadNodes();
          this.selectNode(updated);
        });
    } else {
      // Create
      this.apiService
        .ingestKnowledge(
          this.nodeForm.topic || 'General',
          this.nodeForm.title || '',
          this.nodeForm.description,
          this.nodeForm.tags || [],
          this.nodeForm.content || '',
          this.nodeForm.tuples,
        )
        .subscribe((res) => {
          this.loadNodes();
          this.isEditing = false;
          // Fetch the newly created node to select it
          this.apiService
            .getKnowledgeNode(res.id)
            .subscribe((node) => this.selectNode(node));
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
      this.nodeForm.tags = this.nodeForm.tags.filter((t) => t !== tag);
    }
  }

  // Tuples
  addTuple() {
    if (!this.nodeForm.tuples) {
      this.nodeForm.tuples = [];
    }
    this.nodeForm.tuples.push({
      subject: '',
      predicate: '',
      object: '',
      confidence: 1.0,
    });
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
            chunk_text:
              'Rust enforces memory safety via ownership, borrowing, and lifetime rules without requiring garbage collection.',
            score: 0.94,
          },
        ];
      },
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
          {
            subject: 'SecurityAuditor',
            predicate: 'implements',
            object: 'SecurityTrait',
            confidence: 1.0,
          },
          {
            subject: 'SecurityAuditor',
            predicate: 'uses_tool',
            object: 'RustMemoryScan',
            confidence: 0.95,
          },
        ];
      },
    });
  }
}
