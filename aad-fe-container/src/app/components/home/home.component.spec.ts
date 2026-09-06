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

  it('should define exactly 4 business pillars with unique destination paths', () => {
    expect(component.pillars.length).toBe(4);
    const paths = component.pillars.map(p => p.path);
    expect(paths).toEqual([
      '/knowledge-inspector',
      '/traits',
      '/workbench',
      '/interactive-testing'
    ]);
  });

  it('should represent each destination link strictly once in the page content', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const contentArea = compiled.querySelector('.flex-1') as HTMLElement;
    expect(contentArea).toBeTruthy();

    const checkSingleLink = (path: string) => {
      const allLinks = Array.from(contentArea.querySelectorAll('a'));
      const matchingLinks = allLinks.filter(a => {
        const target = a.getAttribute('routerLink') || a.getAttribute('ng-reflect-router-link') || a.getAttribute('href');
        return target === path || target === `${path}`;
      });
      expect(matchingLinks.length).withContext(`Link for path '${path}' should appear exactly once`).toBe(1);
    };

    checkSingleLink('/knowledge-inspector');
    checkSingleLink('/traits');
    checkSingleLink('/workbench');
    checkSingleLink('/interactive-testing');
    checkSingleLink('/detail');
  });

  it('should explain the Trait concept in plain business terms', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.textContent).toContain('Job Roles & Safety Rules (Traits)');
    expect(compiled.textContent).toContain('Think of Traits like verified job certifications');
    expect(compiled.textContent).toContain('Unbreakable corporate policy rules');
  });

  it('should render the workspace title view switcher dropdown trigger', () => {
    const compiled = fixture.nativeElement as HTMLElement;
    const titleSwitcher = compiled.querySelector('[data-testid="workspace-title-switcher"]');
    expect(titleSwitcher).toBeTruthy();
    expect(titleSwitcher?.textContent).toContain('Agent-As-Data Studio');
  });
});
