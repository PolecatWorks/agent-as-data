import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TraitEmptyComponent } from './trait-empty.component';

describe('TraitEmptyComponent', () => {
  let component: TraitEmptyComponent;
  let fixture: ComponentFixture<TraitEmptyComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TraitEmptyComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(TraitEmptyComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
