import { ComponentFixture, TestBed } from '@angular/core/testing';
import { SkillsRegistryComponent } from './skills-registry.component';
import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { ActivatedRoute } from '@angular/router';
import { of } from 'rxjs';
import { ApiService } from '../../services/api.service';
import { NO_ERRORS_SCHEMA } from '@angular/core';

describe('SkillsRegistryComponent', () => {
  let component: SkillsRegistryComponent;
  let fixture: ComponentFixture<SkillsRegistryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SkillsRegistryComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([]),
        {
          provide: ActivatedRoute,
          useValue: { params: of({}), queryParams: of({}), snapshot: { paramMap: { get: () => null } } }
        },
        {
          provide: ApiService,
          useValue: {
            getSkills: () => of([]),
            getTools: () => of([]),
            getTraits: () => of({ ids: [] }),
            getTrait: () => of(null)
          }
        }
      ],
      schemas: [NO_ERRORS_SCHEMA]
    }).compileComponents();

    fixture = TestBed.createComponent(SkillsRegistryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create the skills registry component', () => {
    expect(component).toBeTruthy();
  });

  it('should render the zero-footprint concept guide trigger and configuration in top bar', () => {
    const trigger = fixture.nativeElement.querySelector('[data-testid="skills-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What are Skills?');

    expect(component.conceptGuideMappings.length).toBe(3);
    expect(component.conceptGuideMappings[0].title).toBe('1. Procedural Instructions');
    expect(component.conceptGuideMappings[1].title).toBe('2. Typed JSON Schemas');
    expect(component.conceptGuideMappings[2].title).toBe('3. Trait Safety Verification');
  });

  it('should render the workspace title as an interactive view switcher trigger with dropdown affordance', () => {
    const switcher = fixture.nativeElement.querySelector('[data-testid="workspace-title-switcher"]');
    expect(switcher).toBeTruthy();
    expect(switcher.textContent).toContain('Skills Registry');
    expect(switcher.textContent).toContain('expand_more');
  });
});

