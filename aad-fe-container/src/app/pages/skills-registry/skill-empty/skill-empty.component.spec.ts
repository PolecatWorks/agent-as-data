import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SkillEmptyComponent } from './skill-empty.component';

describe('SkillEmptyComponent', () => {
  let component: SkillEmptyComponent;
  let fixture: ComponentFixture<SkillEmptyComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SkillEmptyComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(SkillEmptyComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
