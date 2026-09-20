import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter, Router, ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { NO_ERRORS_SCHEMA } from '@angular/core';
import { AgentContextComponent } from './agent-context.component';

describe('AgentContextComponent', () => {
  let component: AgentContextComponent;
  let fixture: ComponentFixture<AgentContextComponent>;
  let router: Router;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentContextComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([]),
        {
          provide: ActivatedRoute,
          useValue: {
            paramMap: of({ get: () => null }),
            queryParams: of({}),
            snapshot: { paramMap: { get: () => null } }
          }
        }
      ],
      schemas: [NO_ERRORS_SCHEMA]
    }).compileComponents();

    router = TestBed.inject(Router);
    spyOn(router, 'navigate');

    fixture = TestBed.createComponent(AgentContextComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render the workspace title view switcher dropdown trigger', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const titleSwitcher = compiled.querySelector('[data-testid="workspace-title-switcher"]');
    expect(titleSwitcher).toBeTruthy();
    expect(titleSwitcher?.textContent).toContain('Agent Context Search');
  });

  it('should render the zero-footprint concept guide for agent context', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const trigger = compiled.querySelector('[data-testid="context-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger?.textContent).toContain('What is Context Search?');
  });

  it('should navigate to /agents/:id when viewDetails is called with an agent result', () => {
    const result = { entity_id: 'agent-123', entity_type: 'agents' };
    component.viewDetails(result);
    expect(router.navigate).toHaveBeenCalledWith(['/agents', 'agent-123']);
  });

  it('should navigate to /skills/:id when viewDetails is called with a skill result', () => {
    const result = { entity_id: 'skill-456', entity_type: 'skills' };
    component.viewDetails(result);
    expect(router.navigate).toHaveBeenCalledWith(['/skills', 'skill-456']);
  });

  it('should render the View Details button, name, match reason, description and omit entity_id', () => {
    component.searchResults = [
      {
        entity_id: 'test-agent-id-12345',
        name: 'Autonomous Research Agent',
        description: 'An agent that performs deep web research and synthesis.',
        entity_type: 'agents',
        field_name: 'description',
        content: 'deep web research',
        score: 0.95,
        match_reason: 'Matched on entity description'
      }
    ];
    fixture.detectChanges();

    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.textContent).toContain('Autonomous Research Agent');
    expect(compiled.textContent).toContain('Matched on entity description');
    expect(compiled.textContent).toContain('An agent that performs deep web research and synthesis.');
    expect(compiled.textContent).not.toContain('test-agent-id-12345');
    expect(compiled.textContent).not.toContain('Semantic similarity');

    const btn = compiled.querySelector('[data-testid="view-details-btn"]') as HTMLButtonElement;
    expect(btn).toBeTruthy();
    expect(btn.textContent).toContain('View Details');

    btn.click();
    expect(router.navigate).toHaveBeenCalledWith(['/agents', 'test-agent-id-12345']);
  });
});
