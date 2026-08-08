import { ComponentFixture, TestBed } from '@angular/core/testing';

import { RefactoringLabComponent } from './refactoring-lab.component';

describe('RefactoringLabComponent', () => {
  let component: RefactoringLabComponent;
  let fixture: ComponentFixture<RefactoringLabComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [RefactoringLabComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(RefactoringLabComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
