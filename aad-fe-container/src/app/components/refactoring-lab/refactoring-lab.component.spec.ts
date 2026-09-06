import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { RefactoringLabComponent } from './refactoring-lab.component';

describe('RefactoringLabComponent', () => {
  let component: RefactoringLabComponent;
  let fixture: ComponentFixture<RefactoringLabComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [RefactoringLabComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([])
      ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(RefactoringLabComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render the workspace title as an interactive view switcher trigger with dropdown affordance', () => {
    const switcher = fixture.nativeElement.querySelector('[data-testid="workspace-title-switcher"]');
    expect(switcher).toBeTruthy();
    expect(switcher.textContent).toContain('Agent Refactoring & Compression Lab');
    expect(switcher.textContent).toContain('expand_more');
  });

  it('should render the zero-footprint concept guide trigger and configuration in top bar', () => {
    const trigger = fixture.nativeElement.querySelector('[data-testid="refactoring-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What is Refactoring Lab?');

    expect(component.conceptGuideMappings.length).toBe(3);
    expect(component.conceptGuideMappings[0].title).toBe('1. Overlap Cluster Detection');
    expect(component.conceptGuideMappings[1].title).toBe('2. Redundant Skill Pruning');
    expect(component.conceptGuideMappings[2].title).toBe('3. Invariant Conflict Analysis');
  });
});

