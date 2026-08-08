import { ComponentFixture, TestBed } from '@angular/core/testing';

import { InteractiveTestingComponent } from './interactive-testing.component';

describe('InteractiveTestingComponent', () => {
  let component: InteractiveTestingComponent;
  let fixture: ComponentFixture<InteractiveTestingComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [InteractiveTestingComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(InteractiveTestingComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
