import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterModule } from '@angular/router';
import { HomeComponent } from './home.component';

describe('HomeComponent', () => {
  let component: HomeComponent;
  let fixture: ComponentFixture<HomeComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HomeComponent, RouterModule.forRoot([])]
    }).compileComponents();

    fixture = TestBed.createComponent(HomeComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should define all 10 core workspace launchpad cards', () => {
    expect(component.workspaceCards.length).toBe(10);
    const paths = component.workspaceCards.map(c => c.path);
    expect(paths).toContain('/workbench');
    expect(paths).toContain('/agents');
    expect(paths).toContain('/traits');
    expect(paths).toContain('/skills');
    expect(paths).toContain('/tools');
    expect(paths).toContain('/interactive-testing');
    expect(paths).toContain('/network-visualizer');
    expect(paths).toContain('/refactoring-lab');
    expect(paths).toContain('/knowledge-inspector');
    expect(paths).toContain('/agent-context');
  });

  it('should define the 5 lifecycle architecture flow phases', () => {
    expect(component.lifecyclePhases.length).toBe(5);
    expect(component.lifecyclePhases[0].title).toContain('Knowledge & Context');
    expect(component.lifecyclePhases[1].title).toContain('Specifications & Contracts');
    expect(component.lifecyclePhases[2].title).toContain('Governance & Topology');
    expect(component.lifecyclePhases[3].title).toContain('Verification & Testing');
    expect(component.lifecyclePhases[4].title).toContain('Workbench Execution');
  });

  it('should define the 4 pillars of the Trait Contract architecture', () => {
    expect(component.traitPillars.length).toBe(4);
    const titles = component.traitPillars.map(p => p.title);
    expect(titles).toContain('Capability Requirements');
    expect(titles).toContain('Behavioral Invariants');
    expect(titles).toContain('Evaluation Rubrics');
    expect(titles).toContain('Inherited Baseline Guardrails');
  });

  it('should render all 10 workspace cards in the DOM with valid routerLink targets', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const cardLinks = compiled.querySelectorAll('[data-testid="workspace-card-link"]');
    expect(cardLinks.length).toBe(10);
  });

  it('should render the Trait Contract deep-dive section with implements vs uses comparison', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const traitSection = compiled.querySelector('[data-testid="trait-deep-dive"]');
    expect(traitSection).toBeTruthy();
    expect(compiled.textContent).toContain('implements_traits');
    expect(compiled.textContent).toContain('uses_traits');
    expect(compiled.textContent).toContain('Behavioral Invariants');
  });

  it('should render the 4 platform operational tenets', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const tenetsSection = compiled.querySelector('[data-testid="platform-tenets"]');
    expect(tenetsSection).toBeTruthy();
    expect(compiled.textContent).toContain('Zero Direct Runtime Env Vars');
    expect(compiled.textContent).toContain('Deterministic Version Lineage');
    expect(compiled.textContent).toContain('Strict Entity Referencing');
    expect(compiled.textContent).toContain('Distributed Run Safety');
  });
});
