import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HttpClientTestingModule } from '@angular/common/http/testing';
import { RouterTestingModule } from '@angular/router/testing';
import { NoopAnimationsModule } from '@angular/platform-browser/animations';
import { TraitsRegistryComponent } from './traits-registry.component';
import { ApiService } from '../../services/api.service';
import { of } from 'rxjs';

describe('TraitsRegistryComponent - Zero-Footprint Concept Guide', () => {
  let component: TraitsRegistryComponent;
  let fixture: ComponentFixture<TraitsRegistryComponent>;
  let apiServiceSpy: jasmine.SpyObj<ApiService>;

  beforeEach(async () => {
    apiServiceSpy = jasmine.createSpyObj('ApiService', [
      'getTraits',
      'getTrait',
      'createTrait',
      'updateTrait',
      'deleteTrait'
    ]);
    apiServiceSpy.getTraits.and.returnValue(of({ ids: [], pagination: { page: 1, size: 10 } }));

    await TestBed.configureTestingModule({
      imports: [
        TraitsRegistryComponent,
        HttpClientTestingModule,
        RouterTestingModule,
        NoopAnimationsModule
      ],
      providers: [
        { provide: ApiService, useValue: apiServiceSpy }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(TraitsRegistryComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create the component', () => {
    expect(component).toBeTruthy();
  });

  it('should render the concept guide trigger button in the top bar', () => {
    const trigger = fixture.nativeElement.querySelector('[data-testid="traits-concept-trigger"]');
    expect(trigger).toBeTruthy();
    expect(trigger.textContent).toContain('What are Traits?');
  });

  it('should keep concept guide closed by default with zero persistent footprint', () => {
    expect(component.isConceptGuideOpen).toBeFalse();
    const popover = fixture.nativeElement.querySelector('[data-testid="traits-concept-popover"]');
    expect(popover).toBeNull();
  });

  it('should toggle and open the floating concept guide popover', () => {
    component.toggleConceptGuide();
    fixture.detectChanges();

    expect(component.isConceptGuideOpen).toBeTrue();
    expect(component.isConceptGuidePinned).toBeTrue();

    const popover = fixture.nativeElement.querySelector('[data-testid="traits-concept-popover"]');
    expect(popover).toBeTruthy();
  });

  it('should explain the Trait concept using the business certification analogy and 3 editor tab mappings', () => {
    component.showConceptGuide();
    fixture.detectChanges();

    const popover = fixture.nativeElement.querySelector('[data-testid="traits-concept-popover"]');
    expect(popover).toBeTruthy();

    const text = popover.textContent;
    expect(text).toContain('Job Roles & Safety Rules (Traits)');
    expect(text).toContain('Think of Traits like verified job certifications');
    expect(text).toContain('Capability Requirements');
    expect(text).toContain('Behavioral Invariants');
    expect(text).toContain('Evaluation Criteria');
  });

  it('should dismiss the popover when the close button is clicked', () => {
    component.toggleConceptGuide();
    fixture.detectChanges();

    expect(component.isConceptGuideOpen).toBeTrue();

    const closeBtn = fixture.nativeElement.querySelector('[data-testid="traits-concept-close"]');
    expect(closeBtn).toBeTruthy();

    closeBtn.click();
    fixture.detectChanges();

    expect(component.isConceptGuideOpen).toBeFalse();
    expect(fixture.nativeElement.querySelector('[data-testid="traits-concept-popover"]')).toBeNull();
  });

  it('should include a direct link to /detail for the architecture blueprint', () => {
    component.showConceptGuide();
    fixture.detectChanges();

    const detailLink = fixture.nativeElement.querySelector('[data-testid="traits-concept-detail-link"]');
    expect(detailLink).toBeTruthy();
    expect(detailLink.getAttribute('routerLink')).toBe('/detail');
  });
});
