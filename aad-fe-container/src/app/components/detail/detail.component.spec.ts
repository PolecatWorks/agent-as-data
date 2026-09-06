import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterModule } from '@angular/router';
import { DetailComponent } from './detail.component';

describe('DetailComponent', () => {
  let component: DetailComponent;
  let fixture: ComponentFixture<DetailComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [DetailComponent, RouterModule.forRoot([])]
    }).compileComponents();

    fixture = TestBed.createComponent(DetailComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should define all 10 core workspace launchpad cards', () => {
    expect(component.workspaceCards.length).toBe(10);
  });

  it('should define the 5 lifecycle architecture flow phases', () => {
    expect(component.lifecyclePhases.length).toBe(5);
  });

  it('should render the Trait Contract deep-dive section with implements vs uses comparison', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const traitSection = compiled.querySelector('[data-testid="trait-deep-dive"]');
    expect(traitSection).toBeTruthy();
    expect(compiled.textContent).toContain('implements_traits');
    expect(compiled.textContent).toContain('uses_traits');
  });

  it('should render the 4 platform operational tenets', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const tenetsSection = compiled.querySelector('[data-testid="platform-tenets"]');
    expect(tenetsSection).toBeTruthy();
  });

  it('should render the workspace title view switcher dropdown trigger', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const titleSwitcher = compiled.querySelector('[data-testid="workspace-title-switcher"]');
    expect(titleSwitcher).toBeTruthy();
    expect(titleSwitcher?.textContent).toContain('System Architecture & Technical Detail');
  });
});
