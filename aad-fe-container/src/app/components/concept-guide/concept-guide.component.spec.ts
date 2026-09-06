import { ComponentFixture, TestBed, fakeAsync, tick } from '@angular/core/testing';
import { ConceptGuideComponent, ConceptTabMapping } from './concept-guide.component';
import { ActivatedRoute, provideRouter } from '@angular/router';

describe('ConceptGuideComponent', () => {
  let component: ConceptGuideComponent;
  let fixture: ComponentFixture<ConceptGuideComponent>;

  const mockTabMappings: ConceptTabMapping[] = [
    {
      icon: 'build',
      iconColor: 'text-indigo-600',
      title: '1. Capability Requirements',
      description: 'Tools, state access, and environment permissions.'
    },
    {
      icon: 'gavel',
      iconColor: 'text-red-500',
      title: '2. Behavioral Invariants',
      description: 'Unbreakable corporate policy rules.'
    },
    {
      icon: 'fact_check',
      iconColor: 'text-emerald-600',
      title: '3. Evaluation Criteria & Guardrails',
      description: 'Data protection and output grading.'
    }
  ];

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ConceptGuideComponent],
      providers: [provideRouter([])]
    }).compileComponents();

    fixture = TestBed.createComponent(ConceptGuideComponent);
    component = fixture.componentInstance;
    component.badge = '2. Enforceable Behavioral Contracts';
    component.title = 'Job Roles & Safety Rules (Traits)';
    component.triggerLabel = 'What are Traits?';
    component.analogyTitle = 'The Hiring Analogy';
    component.analogyText = 'Think of Traits like verified job certifications.';
    component.tabMappings = mockTabMappings;
    component.detailLink = '/detail';
    component.detailLinkLabel = 'Explore Trait Architecture';
    component.testIdPrefix = 'test-concept';
    fixture.detectChanges();
  });

  it('should create the component and be closed by default with zero persistent footprint', () => {
    expect(component).toBeTruthy();
    expect(component.isOpen).toBeFalse();
    expect(component.isPinned).toBeFalse();
    const popover = fixture.nativeElement.querySelector('[data-testid="test-concept-popover"]');
    expect(popover).toBeNull();
  });

  it('should render the trigger button with configured label and icon', () => {
    const trigger = fixture.nativeElement.querySelector('[data-testid="test-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What are Traits?');
  });

  it('should open on show() and display configured badge, title, analogy and mappings', () => {
    component.show();
    fixture.detectChanges();

    expect(component.isOpen).toBeTrue();
    const popover = fixture.nativeElement.querySelector('[data-testid="test-concept-popover"]');
    expect(popover).toBeTruthy();

    const text = popover.textContent;
    expect(text).toContain('2. Enforceable Behavioral Contracts');
    expect(text).toContain('Job Roles & Safety Rules (Traits)');
    expect(text).toContain('The Hiring Analogy');
    expect(text).toContain('Think of Traits like verified job certifications.');
    expect(text).toContain('1. Capability Requirements');
    expect(text).toContain('2. Behavioral Invariants');
    expect(text).toContain('3. Evaluation Criteria & Guardrails');
  });

  it('should toggle and pin popover when clicked', () => {
    component.toggle();
    fixture.detectChanges();

    expect(component.isOpen).toBeTrue();
    expect(component.isPinned).toBeTrue();

    // Toggling again when pinned should close
    component.toggle();
    fixture.detectChanges();

    expect(component.isOpen).toBeFalse();
    expect(component.isPinned).toBeFalse();
  });

  it('should close after delay when hide() is called while unpinned', fakeAsync(() => {
    component.show();
    fixture.detectChanges();
    expect(component.isOpen).toBeTrue();

    component.hide();
    expect(component.isOpen).toBeTrue(); // Not closed immediately (debounced)

    tick(300);
    fixture.detectChanges();
    expect(component.isOpen).toBeFalse();
  }));

  it('should NOT close on hide() when pinned', fakeAsync(() => {
    component.toggle(); // pins
    fixture.detectChanges();
    expect(component.isPinned).toBeTrue();

    component.hide();
    tick(300);
    fixture.detectChanges();

    expect(component.isOpen).toBeTrue();
  }));

  it('should close when close button is clicked and reset pinned state', () => {
    component.toggle();
    fixture.detectChanges();
    expect(component.isOpen).toBeTrue();

    const closeBtn = fixture.nativeElement.querySelector('[data-testid="test-concept-close"]');
    expect(closeBtn).toBeTruthy();

    closeBtn.click();
    fixture.detectChanges();

    expect(component.isOpen).toBeFalse();
    expect(component.isPinned).toBeFalse();
    expect(fixture.nativeElement.querySelector('[data-testid="test-concept-popover"]')).toBeNull();
  });

  it('should close when Escape key is pressed', () => {
    component.toggle();
    fixture.detectChanges();
    expect(component.isOpen).toBeTrue();

    component.onEscapePress();
    fixture.detectChanges();

    expect(component.isOpen).toBeFalse();
  });

  it('should render architecture blueprint link to /detail', () => {
    component.show();
    fixture.detectChanges();

    const detailLink = fixture.nativeElement.querySelector('[data-testid="test-concept-detail-link"]');
    expect(detailLink).toBeTruthy();
    expect(detailLink.getAttribute('href')).toBe('/detail');
    expect(detailLink.textContent).toContain('Explore Trait Architecture');
  });
});
