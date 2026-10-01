import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { of } from 'rxjs';
import { KnowledgeInspectorComponent } from './knowledge-inspector.component';
import { ApiService } from '../../services/api.service';

describe('KnowledgeInspectorComponent', () => {
  let component: KnowledgeInspectorComponent;
  let fixture: ComponentFixture<KnowledgeInspectorComponent>;
  let apiService: ApiService;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [KnowledgeInspectorComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([]),
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(KnowledgeInspectorComponent);
    component = fixture.componentInstance;
    apiService = TestBed.inject(ApiService);

    // Prevent background unhandled HTTP calls in component lifecycle
    spyOn(apiService, 'getKnowledgeNodes').and.returnValue(of([]));
    spyOn(apiService, 'searchKnowledge').and.returnValue(of([]));
    spyOn(apiService, 'traverseGraph').and.returnValue(of([]));

    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render the zero-footprint concept guide trigger and configuration in top bar', () => {
    const trigger = fixture.nativeElement.querySelector(
      '[data-testid="knowledge-concept-trigger"]',
    );
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What is Knowledge Base?');

    expect(component.conceptGuideMappings.length).toBe(3);
    expect(component.conceptGuideMappings[0].title).toBe(
      '1. Semantic Vector Store',
    );
    expect(component.conceptGuideMappings[1].title).toBe(
      '2. Knowledge Graph Triples',
    );
    expect(component.conceptGuideMappings[2].title).toBe(
      '3. Entity Resolution & Pruning',
    );
  });

  it('should render the workspace title as an interactive view switcher trigger with dropdown affordance', () => {
    const switcher = fixture.nativeElement.querySelector(
      '[data-testid="workspace-title-switcher"]',
    );
    expect(switcher).toBeTruthy();
    expect(switcher.textContent).toContain('Knowledge Base');
    expect(switcher.textContent).toContain('expand_more');
  });

  it('should display document summary card and proposals after analyzeMarkdown returns document', () => {
    spyOn(apiService, 'analyzeMarkdown').and.returnValue(
      of({
        document: {
          topic: 'architecture',
          title: 'Platform Overview',
          description: 'Comprehensive overview of platform.',
          tags: ['document', 'imported'],
          content: '# Platform Overview\nContent here',
        },
        proposals: [
          {
            topic: 'storage',
            title: 'Hybrid Store',
            description: 'Postgres pgvector store.',
            tags: ['db'],
            content: 'Details',
          },
        ],
      }),
    );

    component.openMarkdownImport();
    component.markdownInput = '# Platform Overview\nContent here';
    component.analyzeMarkdown();
    fixture.detectChanges();

    expect(component.isAnalyzing).toBeFalse();
    expect(component.importDocumentProposal).toBeTruthy();
    expect(component.importDocumentProposal?.title).toBe('Platform Overview');
    expect(component.importProposals.length).toBe(1);

    const docCard = fixture.nativeElement.querySelector(
      '[data-testid="source-doc-preview-card"]',
    );
    expect(docCard).toBeTruthy();
    const titleInput = docCard.querySelector('input[type="text"]') as HTMLInputElement;
    expect(titleInput.value).toBe('Platform Overview');
  });

  it('should call apiService.importDocument when finalizeImport is called with saveSourceDocument enabled', () => {
    const importSpy = spyOn(apiService, 'importDocument').and.returnValue(
      of({
        document: { id: 'doc-1', title: 'Platform Overview' },
        concepts: [],
        tuples_created: 1,
      }),
    );

    component.openMarkdownImport();
    const docProposal = {
      topic: 'architecture',
      title: 'Platform Overview',
      description: 'Overview',
      tags: ['doc'],
      content: 'Content',
    };
    component.importDocumentProposal = { ...docProposal };
    component.saveSourceDocument = true;
    component.importProposals = [
      {
        topic: 'storage',
        title: 'Hybrid Store',
        description: 'Store',
        tags: [],
        content: 'Content',
        selected: true,
      },
    ];

    component.finalizeImport();

    expect(importSpy).toHaveBeenCalledWith(
      jasmine.objectContaining({ title: 'Platform Overview' }),
      [jasmine.objectContaining({ title: 'Hybrid Store', selected: true })],
      true,
    );
    expect(component.showMarkdownImport).toBeFalse();
  });

  it('should render source document badge and load derived concepts when a source document is selected', () => {
    spyOn(apiService, 'getKnowledgeTuples').and.returnValue(of([]));
    const derivedSpy = spyOn(apiService, 'getDerivedConcepts').and.returnValue(
      of([
        {
          id: 'concept-1',
          title: 'Extracted Concept',
          topic: 'architecture',
          description: 'Desc',
          content: '...',
          tags: [],
          created_at: '',
          updated_at: '',
        } as any,
      ]),
    );

    const sourceDocNode: any = {
      id: 'doc-uuid-1',
      title: 'Master Architecture Doc',
      topic: 'architecture',
      description: 'Summary',
      content: '# Master Doc',
      tags: ['document'],
      metadata: { is_source_document: true },
    };

    component.selectNode(sourceDocNode);
    fixture.detectChanges();

    expect(derivedSpy).toHaveBeenCalledWith('doc-uuid-1');
    expect(component.derivedConcepts.length).toBe(1);

    const badge = fixture.nativeElement.querySelector(
      '[data-testid="source-doc-badge"]',
    );
    expect(badge).toBeTruthy();
    expect(badge.textContent).toContain('Source Document');

    const derivedPanel = fixture.nativeElement.querySelector(
      '[data-testid="derived-concepts-panel"]',
    );
    expect(derivedPanel).toBeTruthy();
    expect(derivedPanel.textContent).toContain('Extracted Concept');
  });

  it('should render derived-from chip when a child concept is selected', () => {
    spyOn(apiService, 'getKnowledgeTuples').and.returnValue(of([]));

    const childConceptNode: any = {
      id: 'concept-uuid-1',
      title: 'Child Concept',
      topic: 'architecture',
      description: 'Desc',
      content: 'Content',
      tags: [],
      metadata: {
        is_extracted_concept: true,
        source_document_id: 'doc-uuid-1',
        source_document_title: 'Master Architecture Doc',
      },
    };

    component.selectNode(childConceptNode);
    fixture.detectChanges();

    const chip = fixture.nativeElement.querySelector(
      '[data-testid="source-doc-chip"]',
    );
    expect(chip).toBeTruthy();
    expect(chip.textContent).toContain('Master Architecture Doc');
  });
});
