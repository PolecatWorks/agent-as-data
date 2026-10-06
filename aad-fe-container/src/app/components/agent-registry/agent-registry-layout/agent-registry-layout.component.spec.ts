import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AgentRegistryLayoutComponent } from './agent-registry-layout.component';

describe('AgentRegistryLayoutComponent', () => {
  let component: AgentRegistryLayoutComponent;
  let fixture: ComponentFixture<AgentRegistryLayoutComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [AgentRegistryLayoutComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(AgentRegistryLayoutComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
