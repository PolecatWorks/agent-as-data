import { ComponentFixture, TestBed } from '@angular/core/testing';
import { EntityPillComponent } from './entity-pill.component';
import { By } from '@angular/platform-browser';

describe('EntityPillComponent', () => {
  let component: EntityPillComponent;
  let fixture: ComponentFixture<EntityPillComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [EntityPillComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(EntityPillComponent);
    component = fixture.componentInstance;
    component.label = 'Test Pill';
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should render label', () => {
    const spanEl = fixture.debugElement.query(By.css('span')).nativeElement;
    expect(spanEl.textContent).toContain('Test Pill');
  });

  it('should emit removed when remove button clicked', () => {
    component.isRemovable = true;
    fixture.detectChanges();
    spyOn(component.removed, 'emit');
    
    const btnEl = fixture.debugElement.query(By.css('button'));
    btnEl.triggerEventHandler('click', new Event('click'));
    
    expect(component.removed.emit).toHaveBeenCalled();
  });
});
