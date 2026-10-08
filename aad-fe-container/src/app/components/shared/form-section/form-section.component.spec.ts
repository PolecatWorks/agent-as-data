import { ComponentFixture, TestBed } from '@angular/core/testing';
import { FormSectionComponent } from './form-section.component';
import { By } from '@angular/platform-browser';

describe('FormSectionComponent', () => {
  let component: FormSectionComponent;
  let fixture: ComponentFixture<FormSectionComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FormSectionComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(FormSectionComponent);
    component = fixture.componentInstance;
    
    // Set required inputs
    component.title = 'Test Title';
    component.icon = 'settings';
    
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
  
  it('should render the title and icon', () => {
    const titleEl = fixture.debugElement.query(By.css('h3')).nativeElement;
    const iconEl = fixture.debugElement.query(By.css('mat-icon')).nativeElement;
    
    expect(titleEl.textContent.trim()).toBe('Test Title');
    expect(iconEl.textContent.trim()).toBe('settings');
  });

  it('should render the description if provided', () => {
    component.description = 'Test Description';
    fixture.detectChanges();
    
    const pEl = fixture.debugElement.query(By.css('p')).nativeElement;
    expect(pEl.textContent.trim()).toBe('Test Description');
  });
});
