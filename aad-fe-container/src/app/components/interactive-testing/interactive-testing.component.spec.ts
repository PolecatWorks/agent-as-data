import { ComponentFixture, TestBed } from '@angular/core/testing';

import { provideHttpClient } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { provideRouter } from '@angular/router';
import { InteractiveTestingComponent } from './interactive-testing.component';

describe('InteractiveTestingComponent', () => {
  let component: InteractiveTestingComponent;
  let fixture: ComponentFixture<InteractiveTestingComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [InteractiveTestingComponent],
      providers: [
        provideHttpClient(),
        provideAnimationsAsync(),
        provideRouter([])
      ]
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
