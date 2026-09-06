import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { KnowledgeInspectorComponent } from './knowledge-inspector.component';

describe('KnowledgeInspectorComponent', () => {
  let component: KnowledgeInspectorComponent;
  let fixture: ComponentFixture<KnowledgeInspectorComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [KnowledgeInspectorComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([])
      ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(KnowledgeInspectorComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render the zero-footprint concept guide trigger and configuration in top bar', () => {
    const trigger = fixture.nativeElement.querySelector('[data-testid="knowledge-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What is Knowledge Base?');

    expect(component.conceptGuideMappings.length).toBe(3);
    expect(component.conceptGuideMappings[0].title).toBe('1. Semantic Vector Store');
    expect(component.conceptGuideMappings[1].title).toBe('2. Knowledge Graph Triples');
    expect(component.conceptGuideMappings[2].title).toBe('3. Entity Resolution & Pruning');
  });

  it('should render the workspace title as an interactive view switcher trigger with dropdown affordance', () => {
    const switcher = fixture.nativeElement.querySelector('[data-testid="workspace-title-switcher"]');
    expect(switcher).toBeTruthy();
    expect(switcher.textContent).toContain('Knowledge & SPO Tuple Inspector');
    expect(switcher.textContent).toContain('expand_more');
  });
});


