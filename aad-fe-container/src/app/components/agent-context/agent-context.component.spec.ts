import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { NO_ERRORS_SCHEMA } from '@angular/core';
import { AgentContextComponent } from './agent-context.component';

describe('AgentContextComponent', () => {
  let component: AgentContextComponent;
  let fixture: ComponentFixture<AgentContextComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentContextComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
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
});
