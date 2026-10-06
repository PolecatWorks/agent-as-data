import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AgentEmptyComponent } from './agent-empty.component';

describe('AgentEmptyComponent', () => {
  let component: AgentEmptyComponent;
  let fixture: ComponentFixture<AgentEmptyComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentEmptyComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(AgentEmptyComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
