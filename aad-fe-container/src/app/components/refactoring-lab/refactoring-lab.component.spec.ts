import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { RefactoringLabComponent } from './refactoring-lab.component';

describe('RefactoringLabComponent', () => {
  let component: RefactoringLabComponent;
  let fixture: ComponentFixture<RefactoringLabComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [RefactoringLabComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([])
      ]
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
